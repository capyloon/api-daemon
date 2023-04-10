//! The client side API
//!
//! The main entry point is [`run`]. This function takes callbacks that will
//! be invoked when blobs or collections are received. It is up to the caller
//! to store the received data.
use std::fmt::Debug;
use std::io;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::blobs::Collection;
use crate::protocol::{
    read_bao_encoded, read_lp, write_lp, AuthToken, Handshake, Request, Res, Response,
};
use crate::provider::Ticket;
use crate::subnet::{same_subnet_v4, same_subnet_v6};
use crate::tls::{self, Keypair, PeerId};
use crate::IROH_BLOCK_SIZE;
use anyhow::{anyhow, bail, Context, Result};
use bao_tree::io::tokio::AsyncResponseDecoder;
use bytes::BytesMut;
use default_net::Interface;
use futures::{Future, StreamExt};
use postcard::experimental::max_size::MaxSize;
use range_collections::RangeSet2;
use tokio::io::{AsyncRead, AsyncReadExt, ReadBuf};
use tracing::{debug, debug_span, error};
use tracing_futures::Instrument;

pub use crate::util::Hash;

/// Options for the client
#[derive(Clone, Debug)]
pub struct Options {
    /// The address to connect to
    pub addr: SocketAddr,
    /// The peer id to expect
    pub peer_id: Option<PeerId>,
    /// Whether to log the SSL keys when `SSLKEYLOGFILE` environment variable is set.
    pub keylog: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            addr: "127.0.0.1:4433".parse().unwrap(),
            peer_id: None,
            keylog: false,
        }
    }
}

/// Create a quinn client endpoint
pub fn make_client_endpoint(
    bind_addr: SocketAddr,
    peer_id: Option<PeerId>,
    alpn_protocols: Vec<Vec<u8>>,
    keylog: bool,
) -> Result<quinn::Endpoint> {
    let keypair = Keypair::generate();

    let tls_client_config = tls::make_client_config(&keypair, peer_id, alpn_protocols, keylog)?;
    let mut client_config = quinn::ClientConfig::new(Arc::new(tls_client_config));
    let mut endpoint = quinn::Endpoint::client(bind_addr)?;
    let mut transport_config = quinn::TransportConfig::default();
    transport_config.keep_alive_interval(Some(Duration::from_secs(1)));
    client_config.transport_config(Arc::new(transport_config));

    endpoint.set_default_client_config(client_config);
    Ok(endpoint)
}

/// Establishes a QUIC connection to the provided peer.
async fn dial_peer(opts: Options) -> Result<quinn::Connection> {
    let bind_addr = match opts.addr.is_ipv6() {
        true => SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, 0, 0, 0).into(),
        false => SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0).into(),
    };
    let endpoint =
        make_client_endpoint(bind_addr, opts.peer_id, vec![tls::P2P_ALPN.to_vec()], false)?;

    debug!("connecting to {}", opts.addr);
    let connect = endpoint.connect(opts.addr, "localhost")?;
    let connection = connect.await.context("failed connecting to provider")?;

    Ok(connection)
}

/// Stats about the transfer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stats {
    /// The number of bytes transferred
    pub data_len: u64,
    /// The time it took to transfer the data
    pub elapsed: Duration,
}

impl Stats {
    /// Transfer rate in megabits per second
    pub fn mbits(&self) -> f64 {
        let data_len_bit = self.data_len * 8;
        data_len_bit as f64 / (1000. * 1000.) / self.elapsed.as_secs_f64()
    }
}

/// A verified stream of data coming from the provider
///
/// We guarantee that the data is correct by incrementally verifying a hash
#[repr(transparent)]
#[derive(Debug)]
pub struct DataStream(AsyncResponseDecoder<quinn::RecvStream>);

impl DataStream {
    fn new(inner: quinn::RecvStream, hash: Hash) -> Self {
        let decoder =
            AsyncResponseDecoder::new(hash.into(), RangeSet2::all(), IROH_BLOCK_SIZE, inner);
        DataStream(decoder)
    }

    async fn read_size(&mut self) -> io::Result<u64> {
        self.0.read_size().await
    }

    fn into_inner(self) -> quinn::RecvStream {
        self.0.into_inner()
    }
}

impl AsyncRead for DataStream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut ReadBuf,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

/// Gets a collection and all its blobs using a [`Ticket`].
pub async fn run_ticket<A, B, C, FutA, FutB, FutC>(
    ticket: &Ticket,
    keylog: bool,
    max_concurrent: u8,
    on_connected: A,
    on_collection: B,
    on_blob: C,
) -> Result<Stats>
where
    A: FnOnce() -> FutA,
    FutA: Future<Output = Result<()>>,
    B: FnOnce(&Collection) -> FutB,
    FutB: Future<Output = Result<()>>,
    C: FnMut(Hash, DataStream, String) -> FutC,
    FutC: Future<Output = Result<DataStream>>,
{
    let span = debug_span!("get", hash=%ticket.hash());
    async move {
        let start = Instant::now();
        let connection = dial_ticket(ticket, keylog, max_concurrent.into()).await?;
        let span = debug_span!("connection", remote_addr=%connection.remote_address());
        run_connection(
            connection,
            ticket.hash(),
            ticket.token(),
            start,
            on_connected,
            on_collection,
            on_blob,
        )
        .instrument(span)
        .await
    }
    .instrument(span)
    .await
}

async fn dial_ticket(
    ticket: &Ticket,
    keylog: bool,
    max_concurrent: usize,
) -> Result<quinn::Connection> {
    // Sort the interfaces to make sure local ones are at the front of the list.
    let interfaces = default_net::get_interfaces();
    let (mut addrs, other_addrs) = ticket
        .addrs()
        .iter()
        .partition::<Vec<_>, _>(|addr| is_same_subnet(addr, &interfaces));
    addrs.extend(other_addrs);

    let mut conn_stream = futures::stream::iter(addrs)
        .map(|addr| {
            let opts = Options {
                addr,
                peer_id: Some(ticket.peer()),
                keylog,
            };
            dial_peer(opts)
        })
        .buffer_unordered(max_concurrent);
    while let Some(res) = conn_stream.next().await {
        match res {
            Ok(conn) => return Ok(conn),
            Err(_) => continue,
        }
    }
    Err(anyhow!("Failed to establish connection to peer"))
}

fn is_same_subnet(addr: &SocketAddr, interfaces: &[Interface]) -> bool {
    for interface in interfaces {
        match addr {
            SocketAddr::V4(peer_addr) => {
                for net in interface.ipv4.iter() {
                    if same_subnet_v4(net.addr, *peer_addr.ip(), net.prefix_len) {
                        return true;
                    }
                }
            }
            SocketAddr::V6(peer_addr) => {
                for net in interface.ipv6.iter() {
                    if same_subnet_v6(net.addr, *peer_addr.ip(), net.prefix_len) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Get a collection and all its blobs from a provider
pub async fn run<A, B, C, FutA, FutB, FutC>(
    hash: Hash,
    auth_token: AuthToken,
    opts: Options,
    on_connected: A,
    on_collection: B,
    on_blob: C,
) -> Result<Stats>
where
    A: FnOnce() -> FutA,
    FutA: Future<Output = Result<()>>,
    B: FnOnce(&Collection) -> FutB,
    FutB: Future<Output = Result<()>>,
    C: FnMut(Hash, DataStream, String) -> FutC,
    FutC: Future<Output = Result<DataStream>>,
{
    let span = debug_span!("get", %hash);
    async move {
        let now = Instant::now();
        let connection = dial_peer(opts).await?;
        let span = debug_span!("connection", remote_addr=%connection.remote_address());
        run_connection(
            connection,
            hash,
            auth_token,
            now,
            on_connected,
            on_collection,
            on_blob,
        )
        .instrument(span)
        .await
    }
    .instrument(span)
    .await
}

/// Gets a collection and all its blobs from a provider on the established connection.
async fn run_connection<A, B, C, FutA, FutB, FutC>(
    connection: quinn::Connection,
    hash: Hash,
    auth_token: AuthToken,
    start_time: Instant,
    on_connected: A,
    on_collection: B,
    mut on_blob: C,
) -> Result<Stats>
where
    A: FnOnce() -> FutA,
    FutA: Future<Output = Result<()>>,
    B: FnOnce(&Collection) -> FutB,
    FutB: Future<Output = Result<()>>,
    C: FnMut(Hash, DataStream, String) -> FutC,
    FutC: Future<Output = Result<DataStream>>,
{
    let (mut writer, mut reader) = connection.open_bi().await?;

    on_connected().await?;

    let mut out_buffer = BytesMut::zeroed(std::cmp::max(
        Request::POSTCARD_MAX_SIZE,
        Handshake::POSTCARD_MAX_SIZE,
    ));

    // 1. Send Handshake
    {
        debug!("sending handshake");
        let handshake = Handshake::new(auth_token);
        let used = postcard::to_slice(&handshake, &mut out_buffer)?;
        write_lp(&mut writer, used).await?;
    }

    // 2. Send Request
    {
        debug!("sending request");
        let req = Request { name: hash };

        let used = postcard::to_slice(&req, &mut out_buffer)?;
        write_lp(&mut writer, used).await?;
    }
    writer.finish().await?;
    drop(writer);

    // 3. Read response
    {
        debug!("reading response");
        let mut in_buffer = BytesMut::with_capacity(1024);

        // track total amount of blob data transferred
        let mut data_len = 0;
        // read next message
        match read_lp(&mut reader, &mut in_buffer).await? {
            Some(response_buffer) => {
                let response: Response = postcard::from_bytes(&response_buffer)?;
                match response.data {
                    // server is sending over a collection of blobs
                    Res::FoundCollection { total_blobs_size } => {
                        data_len = total_blobs_size;

                        // read entire collection data into buffer
                        let data = read_bao_encoded(&mut reader, hash).await?;

                        // decode the collection
                        let collection = Collection::from_bytes(&data)?;
                        on_collection(&collection).await?;

                        // expect to get blob data in the order they appear in the collection
                        let mut remaining_size = total_blobs_size;
                        for blob in collection.into_inner() {
                            let mut blob_reader =
                                handle_blob_response(blob.hash, reader, &mut in_buffer).await?;

                            let size = blob_reader.read_size().await?;
                            anyhow::ensure!(
                                size <= remaining_size,
                                "downloaded more than {total_blobs_size}"
                            );
                            remaining_size -= size;
                            let mut blob_reader =
                                on_blob(blob.hash, blob_reader, blob.name).await?;

                            if blob_reader.read_exact(&mut [0u8; 1]).await.is_ok() {
                                bail!("`on_blob` callback did not fully read the blob content")
                            }
                            reader = blob_reader.into_inner();
                        }
                    }

                    // unexpected message
                    Res::Found { .. } => {
                        // we should only receive `Res::FoundCollection` or `Res::NotFound` from the
                        // provider at this point in the exchange
                        bail!("Unexpected message from provider. Ending transfer early.");
                    }

                    // data associated with the hash is not found
                    Res::NotFound => {
                        Err(anyhow!("data not found"))?;
                    }
                }

                // Shut down the stream
                if let Some(chunk) = reader.read_chunk(8, false).await? {
                    reader.stop(0u8.into()).ok();
                    error!("Received unexpected data from the provider: {chunk:?}");
                }
                drop(reader);

                let elapsed = start_time.elapsed();

                let stats = Stats { data_len, elapsed };

                Ok(stats)
            }
            None => {
                bail!("provider closed stream");
            }
        }
    }
}

/// Gets only the first blob in a collection from a provider on the established connection.
pub async fn get_first_blob(
    ticket: &Ticket,
    keylog: bool,
    max_concurrent: u8,
) -> Result<(DataStream, Option<String>, u64)> {
    let connection = dial_ticket(ticket, keylog, max_concurrent.into()).await?;
    let (mut writer, mut reader) = connection.open_bi().await?;

    let mut out_buffer = BytesMut::zeroed(std::cmp::max(
        Request::POSTCARD_MAX_SIZE,
        Handshake::POSTCARD_MAX_SIZE,
    ));

    // 1. Send Handshake
    {
        debug!("sending handshake");
        let handshake = Handshake::new(ticket.token());
        let used = postcard::to_slice(&handshake, &mut out_buffer)?;
        write_lp(&mut writer, used).await?;
    }

    // 2. Send Request
    {
        debug!("sending request");
        let req = Request {
            name: ticket.hash(),
        };

        let used = postcard::to_slice(&req, &mut out_buffer)?;
        write_lp(&mut writer, used).await?;
    }
    writer.finish().await?;
    drop(writer);

    // 3. Read response
    {
        debug!("reading response");
        let mut in_buffer = BytesMut::with_capacity(1024);

        // read next message
        match read_lp(&mut reader, &mut in_buffer).await? {
            Some(response_buffer) => {
                let response: Response = postcard::from_bytes(&response_buffer)?;
                match response.data {
                    // server is sending over a collection of blobs
                    Res::FoundCollection { .. } => {
                        // read entire collection data into buffer
                        let data = read_bao_encoded(&mut reader, ticket.hash()).await?;

                        // decode the collection
                        let collection = Collection::from_bytes(&data)?;
                        if collection.total_entries() != 1 {
                            bail!("Only single blob collections are supported.");
                        }

                        // expect to get blob data in the order they appear in the collection
                        let blob = collection.into_inner().pop().unwrap();

                        let mut blob_reader =
                            handle_blob_response(blob.hash, reader, &mut in_buffer).await?;

                        let size = blob_reader.read_size().await?;
                        Ok((blob_reader, blob.mime, size))
                    }

                    // unexpected message
                    Res::Found { .. } => {
                        // we should only receive `Res::FoundCollection` or `Res::NotFound` from the
                        // provider at this point in the exchange
                        bail!("Unexpected message from provider. Ending transfer early.");
                    }

                    // data associated with the hash is not found
                    Res::NotFound => Err(anyhow!("data not found")),
                }
            }
            None => {
                bail!("provider closed stream");
            }
        }
    }
}

/// Read next response, and if `Res::Found`, reads the next blob of data off the reader.
///
/// Returns an `AsyncReader`
/// The `AsyncReader` can be used to read the content.
async fn handle_blob_response(
    hash: Hash,
    mut reader: quinn::RecvStream,
    buffer: &mut BytesMut,
) -> Result<DataStream> {
    match read_lp(&mut reader, buffer).await? {
        Some(response_buffer) => {
            let response: Response = postcard::from_bytes(&response_buffer)?;
            match response.data {
                // unexpected message
                Res::FoundCollection { .. } => Err(anyhow!(
                    "Unexpected message from provider. Ending transfer early."
                ))?,
                // blob data not found
                Res::NotFound => Err(anyhow!("data for {} not found", hash))?,
                // next blob in collection will be sent over
                Res::Found => {
                    assert!(buffer.is_empty());
                    let decoder = DataStream::new(reader, hash);
                    Ok(decoder)
                }
            }
        }
        None => Err(anyhow!("server disconnected"))?,
    }
}
