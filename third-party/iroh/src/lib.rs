//! Send data over the internet.
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
pub mod blobs;
pub mod get;
#[cfg(feature = "metrics")]
pub mod metrics;
pub mod net;
pub mod progress;
pub mod protocol;
pub mod provider;
pub mod rpc_protocol;

mod subnet;
mod tls;
mod util;

pub use tls::{Keypair, PeerId, PeerIdError, PublicKey, SecretKey, Signature};
pub use util::Hash;

use bao_tree::BlockSize;

pub(crate) const IROH_BLOCK_SIZE: BlockSize = match BlockSize::new(4) {
    Some(bs) => bs,
    None => panic!(),
};

#[cfg(test)]
mod tests {
    use std::{
        net::{Ipv4Addr, SocketAddr},
        path::{Path, PathBuf},
        sync::{atomic::AtomicUsize, Arc},
        time::Duration,
    };

    use anyhow::{anyhow, Context, Result};
    use rand::RngCore;
    use testdir::testdir;
    use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
    use tokio::{fs, sync::broadcast};
    use tracing_subscriber::{prelude::*, EnvFilter};

    use crate::protocol::AuthToken;
    use crate::provider::{create_collection, Event, Provider};
    use crate::tls::PeerId;
    use crate::util::Hash;

    use super::*;

    #[tokio::test]
    async fn basics() -> Result<()> {
        transfer_data(vec![("hello_world", "hello world!".as_bytes().to_vec())]).await
    }

    #[tokio::test]
    async fn multi_file() -> Result<()> {
        let file_opts = vec![
            ("1", 10),
            ("2", 1024),
            ("3", 1024 * 1024),
            // overkill, but it works! Just annoying to wait for
            // ("4", 1024 * 1024 * 90),
        ];
        transfer_random_data(file_opts).await
    }

    #[tokio::test]
    async fn many_files() -> Result<()> {
        setup_logging();
        let num_files = [10, 100, 1000, 10000];
        for num in num_files {
            println!("NUM_FILES: {num}");
            let file_opts = (0..num)
                .map(|i| {
                    // use a long file name to test large collections
                    let name = i.to_string().repeat(50);
                    (name, 10)
                })
                .collect();
            transfer_random_data(file_opts).await?;
        }
        Ok(())
    }

    #[tokio::test]
    async fn sizes() -> Result<()> {
        let sizes = [
            0,
            10,
            100,
            1024,
            1024 * 100,
            1024 * 500,
            1024 * 1024,
            1024 * 1024 + 10,
        ];

        for size in sizes {
            transfer_random_data(vec![("hello_world", size)]).await?;
        }

        Ok(())
    }

    #[tokio::test]
    async fn empty_files() -> Result<()> {
        // try to transfer as many files as possible without hitting a limit
        // booo 400 is too small :(
        let num_files = 400;
        let mut file_opts = Vec::new();
        for i in 0..num_files {
            file_opts.push((i.to_string(), 0));
        }
        transfer_random_data(file_opts).await
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn multiple_clients() -> Result<()> {
        let dir: PathBuf = testdir!();
        let filename = "hello_world";
        let path = dir.join(filename);
        let content = b"hello world!";
        let addr = "127.0.0.1:0".parse().unwrap();

        tokio::fs::write(&path, content).await?;
        // hash of the transfer file
        let data = tokio::fs::read(&path).await?;
        let expect_hash = blake3::hash(&data);
        let expect_name = filename.to_string();

        let (db, hash) =
            provider::create_collection(vec![provider::DataSource::File((path, None))]).await?;
        let provider = provider::Provider::builder(db).bind_addr(addr).spawn()?;

        async fn run_client(
            hash: Hash,
            token: AuthToken,
            file_hash: Hash,
            name: String,
            addr: SocketAddr,
            peer_id: PeerId,
            content: Vec<u8>,
        ) -> Result<()> {
            let opts = get::Options {
                addr,
                peer_id: Some(peer_id),
                keylog: true,
            };
            let content = &content;
            let name = &name;
            get::run(
                hash,
                token,
                opts,
                || async { Ok(()) },
                |_collection| async { Ok(()) },
                |got_hash, mut reader, got_name| async move {
                    assert_eq!(file_hash, got_hash);
                    let mut got = Vec::new();
                    reader.read_to_end(&mut got).await?;
                    assert_eq!(content, &got);
                    assert_eq!(*name, got_name);

                    Ok(reader)
                },
            )
            .await?;

            Ok(())
        }

        let mut tasks = Vec::new();
        for _i in 0..3 {
            tasks.push(tokio::task::spawn(run_client(
                hash,
                provider.auth_token(),
                expect_hash.into(),
                expect_name.clone(),
                provider.local_address(),
                provider.peer_id(),
                content.to_vec(),
            )));
        }

        futures::future::join_all(tasks).await;

        Ok(())
    }

    // Run the test creating random data for each blob, using the size specified by the file
    // options
    async fn transfer_random_data<S>(file_opts: Vec<(S, usize)>) -> Result<()>
    where
        S: Into<String> + std::fmt::Debug + std::cmp::PartialEq,
    {
        let file_opts = file_opts
            .into_iter()
            .map(|(name, size)| {
                let mut content = vec![0u8; size];
                rand::thread_rng().fill_bytes(&mut content);
                (name, content)
            })
            .collect();
        transfer_data(file_opts).await
    }

    // Run the test for a vec of filenames and blob data
    async fn transfer_data<S>(file_opts: Vec<(S, Vec<u8>)>) -> Result<()>
    where
        S: Into<String> + std::fmt::Debug + std::cmp::PartialEq,
    {
        let dir: PathBuf = testdir!();

        // create and save files
        let mut files = Vec::new();
        let mut expects = Vec::new();
        let num_blobs = file_opts.len();

        for opt in file_opts.into_iter() {
            let (name, data) = opt;

            let name = name.into();
            let path = dir.join(name.clone());
            // get expected hash of file
            let hash = blake3::hash(&data);
            let hash = Hash::from(hash);

            tokio::fs::write(&path, data).await?;
            files.push(provider::DataSource::File((path.clone(), None)));

            // keep track of expected values
            expects.push((name, path, hash));
        }
        // sort expects by name to match the canonical order of blobs
        expects.sort_by(|a, b| a.0.cmp(&b.0));

        let (db, collection_hash) = provider::create_collection(files).await?;

        let addr = "127.0.0.1:0".parse().unwrap();
        let provider = provider::Provider::builder(db).bind_addr(addr).spawn()?;
        let mut provider_events = provider.subscribe();
        let events_task = tokio::task::spawn(async move {
            let mut events = Vec::new();
            loop {
                match provider_events.recv().await {
                    Ok(event) => match event {
                        Event::TransferCollectionCompleted { .. }
                        | Event::TransferAborted { .. } => {
                            events.push(event);
                            break;
                        }
                        _ => events.push(event),
                    },
                    Err(e) => match e {
                        broadcast::error::RecvError::Closed => {
                            break;
                        }
                        broadcast::error::RecvError::Lagged(num) => {
                            panic!("unable to keep up, skipped {num} messages");
                        }
                    },
                }
            }
            events
        });

        let opts = get::Options {
            addr: dbg!(provider.local_address()),
            peer_id: Some(provider.peer_id()),
            keylog: true,
        };

        let i = AtomicUsize::new(0);
        let expects = Arc::new(expects);

        get::run(
            collection_hash,
            provider.auth_token(),
            opts,
            || async { Ok(()) },
            |collection| {
                assert_eq!(collection.blobs().len(), num_blobs);
                async { Ok(()) }
            },
            |got_hash, mut reader, got_name| {
                let i = &i;
                let expects = expects.clone();
                async move {
                    let iv = i.load(std::sync::atomic::Ordering::SeqCst);
                    let (expect_name, path, expect_hash) = expects.get(iv).unwrap();
                    assert_eq!(*expect_hash, got_hash);
                    let expect = tokio::fs::read(&path).await?;
                    let mut got = Vec::new();
                    reader.read_to_end(&mut got).await?;
                    assert_eq!(expect, got);
                    assert_eq!(*expect_name, got_name);
                    i.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    Ok(reader)
                }
            },
        )
        .await?;

        // We have to wait for the completed event before shutting down the provider.
        let events = tokio::time::timeout(Duration::from_secs(30), events_task)
            .await
            .expect("duration expired")
            .expect("events task failed");
        provider.shutdown();
        provider.await?;

        assert_events(events, num_blobs);

        Ok(())
    }

    fn assert_events(events: Vec<Event>, num_blobs: usize) {
        let num_basic_events = 4;
        let num_total_events = num_basic_events + num_blobs;
        assert_eq!(
            events.len(),
            num_total_events,
            "missing events, only got {:#?}",
            events
        );
        assert!(matches!(events[0], Event::ClientConnected { .. }));
        assert!(matches!(events[1], Event::RequestReceived { .. }));
        assert!(matches!(events[2], Event::TransferCollectionStarted { .. }));
        for (i, event) in events[3..num_total_events - 1].iter().enumerate() {
            match event {
                Event::TransferBlobCompleted { index, .. } => {
                    assert_eq!(*index, i as u64);
                }
                _ => panic!("unexpected event {:?}", event),
            }
        }
        assert!(matches!(
            events.last().unwrap(),
            Event::TransferCollectionCompleted { .. }
        ));
    }

    fn setup_logging() {
        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
            .with(EnvFilter::from_default_env())
            .try_init()
            .ok();
    }

    #[tokio::test]
    async fn test_server_close() {
        // Prepare a Provider transferring a file.
        setup_logging();
        let dir = testdir!();
        let src = dir.join("src");
        fs::write(&src, "hello there").await.unwrap();
        let (db, hash) = create_collection(vec![src.into()]).await.unwrap();
        let mut provider = Provider::builder(db)
            .bind_addr("127.0.0.1:0".parse().unwrap())
            .spawn()
            .unwrap();
        let auth_token = provider.auth_token();
        let provider_addr = provider.local_address();

        // This tasks closes the connection on the provider side as soon as the transfer
        // completes.
        let supervisor = tokio::spawn(async move {
            let mut events = provider.subscribe();
            loop {
                tokio::select! {
                    biased;
                    res = &mut provider => break res.context("provider failed"),
                    maybe_event = events.recv() => {
                        match maybe_event {
                            Ok(event) => {
                                match event {
                                    Event::TransferCollectionCompleted { .. } => provider.shutdown(),
                                    Event::TransferAborted { .. } => {
                                        break Err(anyhow!("transfer aborted"));
                                    }
                                    _ => (),
                                }
                            }
                            Err(err) => break Err(anyhow!("event failed: {err:#}")),
                        }
                    }
                }
            }
        });

        get::run(
            hash,
            auth_token,
            get::Options {
                addr: provider_addr,
                peer_id: None,
                keylog: true,
            },
            || async move { Ok(()) },
            |_collection| async move { Ok(()) },
            |_hash, mut stream, _name| async move {
                io::copy(&mut stream, &mut io::sink()).await?;
                Ok(stream)
            },
        )
        .await
        .unwrap();

        // Unwrap the JoinHandle, then the result of the Provider
        tokio::time::timeout(Duration::from_secs(10), supervisor)
            .await
            .expect("supervisor timeout")
            .expect("supervisor failed")
            .expect("supervisor error");
    }

    #[tokio::test]
    async fn test_blob_reader_partial() -> Result<()> {
        // Prepare a Provider transferring a file.
        let dir = testdir!();
        let src0 = dir.join("src0");
        let src1 = dir.join("src1");
        {
            let content = vec![1u8; 1000];
            let mut f = tokio::fs::File::create(&src0).await?;
            for _ in 0..10 {
                f.write_all(&content).await?;
            }
        }
        fs::write(&src1, "hello world").await?;
        let (db, hash) = create_collection(vec![src0.into(), src1.into()]).await?;
        let provider = Provider::builder(db)
            .bind_addr("127.0.0.1:0".parse().unwrap())
            .spawn()?;
        let auth_token = provider.auth_token();
        let provider_addr = provider.local_address();

        let timeout = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            get::run(
                hash,
                auth_token,
                get::Options {
                    addr: provider_addr,
                    peer_id: None,
                    keylog: true,
                },
                || async move { Ok(()) },
                |_collection| async move { Ok(()) },
                |_hash, stream, _name| async move {
                    // evil: do nothing with the stream!
                    Ok(stream)
                },
            ),
        )
        .await;
        provider.shutdown();

        let err = timeout.expect(
            "`get` function is hanging, make sure we are handling misbehaving `on_blob` functions",
        );

        err.expect_err("expected an error when passing in a misbehaving `on_blob` function");
        Ok(())
    }

    #[tokio::test]
    async fn test_ipv6() {
        let readme = Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md");
        let (db, hash) = create_collection(vec![readme.into()]).await.unwrap();
        let provider = match Provider::builder(db)
            .bind_addr("[::1]:0".parse().unwrap())
            .spawn()
        {
            Ok(provider) => provider,
            Err(_) => {
                // We assume the problem here is IPv6 on this host.  If the problem is
                // not IPv6 then other tests will also fail.
                return;
            }
        };
        let auth_token = provider.auth_token();
        let addr = provider.local_address();
        let peer_id = Some(provider.peer_id());
        tokio::time::timeout(
            Duration::from_secs(10),
            get::run(
                hash,
                auth_token,
                get::Options {
                    addr,
                    peer_id,
                    keylog: true,
                },
                || async move { Ok(()) },
                |_collection| async move { Ok(()) },
                |_hash, mut stream, _name| async move {
                    io::copy(&mut stream, &mut io::sink()).await?;
                    Ok(stream)
                },
            ),
        )
        .await
        .expect("timeout")
        .expect("get failed");
    }

    #[tokio::test]
    async fn test_run_ticket() {
        let readme = Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md");
        let (db, hash) = create_collection(vec![readme.into()]).await.unwrap();
        let provider = Provider::builder(db)
            .bind_addr((Ipv4Addr::UNSPECIFIED, 0).into())
            .spawn()
            .unwrap();
        let _drop_guard = provider.cancel_token().drop_guard();
        let ticket = provider.ticket(hash).unwrap();
        let mut on_connected = false;
        let mut on_collection = false;
        let mut on_blob = false;
        tokio::time::timeout(
            Duration::from_secs(10),
            get::run_ticket(
                &ticket,
                true,
                16,
                || {
                    on_connected = true;
                    async { Ok(()) }
                },
                |_| {
                    on_collection = true;
                    async { Ok(()) }
                },
                |_hash, mut stream, _name| {
                    on_blob = true;
                    async move {
                        io::copy(&mut stream, &mut io::sink()).await?;
                        Ok(stream)
                    }
                },
            ),
        )
        .await
        .expect("timeout")
        .expect("get ticket failed");
        assert!(on_connected);
        assert!(on_collection);
        assert!(on_blob);
    }
}
