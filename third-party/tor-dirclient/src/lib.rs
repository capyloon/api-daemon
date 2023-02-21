#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
//! `tor-dirclient`: Implements a minimal directory client for Tor.
//!
//! # Overview
//!
//! Tor makes its directory requests as HTTP/1.0 requests tunneled over
//! Tor circuits.  For most objects, Tor uses a one-hop tunnel.  Tor
//! also uses a few strange and ad-hoc HTTP headers to select
//! particular functionality, such as asking for diffs, compression,
//! or multiple documents.
//!
//! This crate provides an API for downloading Tor directory resources
//! over a Tor circuit.
//!
//! This crate is part of
//! [Arti](https://gitlab.torproject.org/tpo/core/arti/), a project to
//! implement [Tor](https://www.torproject.org/) in Rust.
//!
//! # Features
//!
//! `xz` -- enable XZ compression.  This can be expensive in RAM and CPU,
//! but it saves a lot of bandwidth.  (On by default.)
//!
//! `zstd` -- enable ZSTD compression.  (On by default.)
//!
//! `routerdesc` -- Add support for downloading router descriptors.

// @@ begin lint list maintained by maint/add_warning @@
#![cfg_attr(not(ci_arti_stable), allow(renamed_and_removed_lints))]
#![cfg_attr(not(ci_arti_nightly), allow(unknown_lints))]
#![deny(missing_docs)]
#![warn(noop_method_call)]
#![deny(unreachable_pub)]
#![warn(clippy::all)]
#![deny(clippy::await_holding_lock)]
#![deny(clippy::cargo_common_metadata)]
#![deny(clippy::cast_lossless)]
#![deny(clippy::checked_conversions)]
#![warn(clippy::cognitive_complexity)]
#![deny(clippy::debug_assert_with_mut_call)]
#![deny(clippy::exhaustive_enums)]
#![deny(clippy::exhaustive_structs)]
#![deny(clippy::expl_impl_clone_on_copy)]
#![deny(clippy::fallible_impl_from)]
#![deny(clippy::implicit_clone)]
#![deny(clippy::large_stack_arrays)]
#![warn(clippy::manual_ok_or)]
#![deny(clippy::missing_docs_in_private_items)]
#![deny(clippy::missing_panics_doc)]
#![warn(clippy::needless_borrow)]
#![warn(clippy::needless_pass_by_value)]
#![warn(clippy::option_option)]
#![warn(clippy::rc_buffer)]
#![deny(clippy::ref_option_ref)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![warn(clippy::trait_duplication_in_bounds)]
#![deny(clippy::unnecessary_wraps)]
#![warn(clippy::unseparated_literal_suffix)]
#![deny(clippy::unwrap_used)]
#![allow(clippy::let_unit_value)] // This can reasonably be done for explicitness
#![allow(clippy::significant_drop_in_scrutinee)] // arti/-/merge_requests/588/#note_2812945
//! <!-- @@ end lint list maintained by maint/add_warning @@ -->

mod err;
pub mod request;
mod response;
mod util;

use tor_circmgr::{CircMgr, DirInfo};
use tor_rtcompat::{Runtime, SleepProvider, SleepProviderExt};

// Zlib is required; the others are optional.
#[cfg(feature = "xz")]
use async_compression::futures::bufread::XzDecoder;
use async_compression::futures::bufread::ZlibDecoder;
#[cfg(feature = "zstd")]
use async_compression::futures::bufread::ZstdDecoder;

use futures::io::{
    AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader,
};
use futures::FutureExt;
use memchr::memchr;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

pub use err::{Error, RequestError};
pub use response::{DirResponse, SourceInfo};

/// Type for results returned in this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Type for internal results  containing a RequestError.
pub type RequestResult<T> = std::result::Result<T, RequestError>;

/// Fetch the resource described by `req` over the Tor network.
///
/// Circuits are built or found using `circ_mgr`, using paths
/// constructed using `dirinfo`.
///
/// For more fine-grained control over the circuit and stream used,
/// construct them yourself, and then call [`download`] instead.
///
/// # TODO
///
/// This is the only function in this crate that knows about CircMgr and
/// DirInfo.  Perhaps this function should move up a level into DirMgr?
pub async fn get_resource<CR, R, SP>(
    req: &CR,
    dirinfo: DirInfo<'_>,
    runtime: &SP,
    circ_mgr: Arc<CircMgr<R>>,
) -> Result<DirResponse>
where
    CR: request::Requestable + ?Sized,
    R: Runtime,
    SP: SleepProvider,
{
    let circuit = circ_mgr.get_or_launch_dir(dirinfo).await?;

    // TODO(nickm) This should be an option, and is too long.
    let begin_timeout = Duration::from_secs(5);
    let source = SourceInfo::from_circuit(&circuit);

    let wrap_err = |error| Error::RequestFailed {
        source: Some(source.clone()),
        error,
    };

    req.check_circuit(&circuit).map_err(wrap_err)?;

    // Launch the stream.
    let mut stream = runtime
        .timeout(begin_timeout, circuit.begin_dir_stream())
        .await
        .map_err(RequestError::from)
        .map_err(wrap_err)?
        .map_err(RequestError::from)
        .map_err(wrap_err)?; // TODO(nickm) handle fatalities here too

    // TODO: Perhaps we want separate timeouts for each phase of this.
    // For now, we just use higher-level timeouts in `dirmgr`.
    let r = download(runtime, req, &mut stream, Some(source.clone())).await;

    if should_retire_circ(&r) {
        retire_circ(&circ_mgr, &source, "Partial response");
    }

    r
}

/// Return true if `result` holds an error indicating that we should retire the
/// circuit used for the corresponding request.
fn should_retire_circ(result: &Result<DirResponse>) -> bool {
    match result {
        Err(e) => e.should_retire_circ(),
        Ok(dr) => dr.error().map(RequestError::should_retire_circ) == Some(true),
    }
}

/// Fetch a Tor directory object from a provided stream.
///
/// To do this, we send a simple HTTP/1.0 request for the described
/// object in `req` over `stream`, and then wait for a response.  In
/// log messages, we describe the origin of the data as coming from
/// `source`.
///
/// # Notes
///
/// It's kind of bogus to have a 'source' field here at all; we may
/// eventually want to remove it.
///
/// This function doesn't close the stream; you may want to do that
/// yourself.
pub async fn download<R, S, SP>(
    runtime: &SP,
    req: &R,
    stream: &mut S,
    source: Option<SourceInfo>,
) -> Result<DirResponse>
where
    R: request::Requestable + ?Sized,
    S: AsyncRead + AsyncWrite + Send + Unpin,
    SP: SleepProvider,
{
    let wrap_err = |error| Error::RequestFailed {
        source: source.clone(),
        error,
    };

    let partial_ok = req.partial_docs_ok();
    let maxlen = req.max_response_len();
    let req = req.make_request().map_err(wrap_err)?;
    let encoded = util::encode_request(&req);

    // Write the request.
    stream
        .write_all(encoded.as_bytes())
        .await
        .map_err(RequestError::from)
        .map_err(wrap_err)?;
    stream
        .flush()
        .await
        .map_err(RequestError::from)
        .map_err(wrap_err)?;

    let mut buffered = BufReader::new(stream);

    // Handle the response
    // TODO: should there be a separate timeout here?
    let header = read_headers(&mut buffered).await.map_err(wrap_err)?;
    if header.status != Some(200) {
        return Ok(DirResponse::new(
            header.status.unwrap_or(0),
            None,
            vec![],
            source,
        ));
    }

    let mut decoder = get_decoder(buffered, header.encoding.as_deref()).map_err(wrap_err)?;

    let mut result = Vec::new();
    let ok = read_and_decompress(runtime, &mut decoder, maxlen, &mut result).await;

    let ok = match (partial_ok, ok, result.len()) {
        (true, Err(e), n) if n > 0 => {
            // Note that we _don't_ return here: we want the partial response.
            Err(e)
        }
        (_, Err(e), _) => {
            return Err(wrap_err(e));
        }
        (_, Ok(()), _) => Ok(()),
    };

    Ok(DirResponse::new(200, ok.err(), result, source))
}

/// Read and parse HTTP/1 headers from `stream`.
async fn read_headers<S>(stream: &mut S) -> RequestResult<HeaderStatus>
where
    S: AsyncBufRead + Unpin,
{
    let mut buf = Vec::with_capacity(1024);

    loop {
        // TODO: it's inefficient to do this a line at a time; it would
        // probably be better to read until the CRLF CRLF ending of the
        // response.  But this should be fast enough.
        let n = read_until_limited(stream, b'\n', 2048, &mut buf).await?;

        // TODO(nickm): Better maximum and/or let this expand.
        let mut headers = [httparse::EMPTY_HEADER; 32];
        let mut response = httparse::Response::new(&mut headers);

        match response.parse(&buf[..])? {
            httparse::Status::Partial => {
                // We didn't get a whole response; we may need to try again.

                if n == 0 {
                    // We hit an EOF; no more progress can be made.
                    return Err(RequestError::TruncatedHeaders);
                }

                // TODO(nickm): Pick a better maximum
                if buf.len() >= 16384 {
                    return Err(httparse::Error::TooManyHeaders.into());
                }
            }
            httparse::Status::Complete(n_parsed) => {
                if response.code != Some(200) {
                    return Ok(HeaderStatus {
                        status: response.code,
                        encoding: None,
                    });
                }
                let encoding = if let Some(enc) = response
                    .headers
                    .iter()
                    .find(|h| h.name == "Content-Encoding")
                {
                    Some(String::from_utf8(enc.value.to_vec())?)
                } else {
                    None
                };
                /*
                if let Some(clen) = response.headers.iter().find(|h| h.name == "Content-Length") {
                    let clen = std::str::from_utf8(clen.value)?;
                    length = Some(clen.parse()?);
                }
                 */
                assert!(n_parsed == buf.len());
                return Ok(HeaderStatus {
                    status: Some(200),
                    encoding,
                });
            }
        }
        if n == 0 {
            return Err(RequestError::TruncatedHeaders);
        }
    }
}

/// Return value from read_headers
#[derive(Debug, Clone)]
struct HeaderStatus {
    /// HTTP status code.
    status: Option<u16>,
    /// The Content-Encoding header, if any.
    encoding: Option<String>,
}

/// Helper: download directory information from `stream` and
/// decompress it into a result buffer.  Assumes that `buf` is empty.
///
/// If we get more than maxlen bytes after decompression, give an error.
///
/// Returns the status of our download attempt, stores any data that
/// we were able to download into `result`.  Existing contents of
/// `result` are overwritten.
async fn read_and_decompress<S, SP>(
    runtime: &SP,
    mut stream: S,
    maxlen: usize,
    result: &mut Vec<u8>,
) -> RequestResult<()>
where
    S: AsyncRead + Unpin,
    SP: SleepProvider,
{
    let buffer_window_size = 1024;
    let mut written_total: usize = 0;
    // TODO(nickm): This should be an option, and is maybe too long.
    // Though for some users it may be too short?
    let read_timeout = Duration::from_secs(10);
    let timer = runtime.sleep(read_timeout).fuse();
    futures::pin_mut!(timer);

    loop {
        // allocate buffer for next read
        result.resize(written_total + buffer_window_size, 0);
        let buf: &mut [u8] = &mut result[written_total..written_total + buffer_window_size];

        let status = futures::select! {
            status = stream.read(buf).fuse() => status,
            _ = timer => {
                result.resize(written_total, 0); // truncate as needed
                return Err(RequestError::DirTimeout);
            }
        };
        let written_in_this_loop = match status {
            Ok(n) => n,
            Err(other) => {
                result.resize(written_total, 0); // truncate as needed
                return Err(other.into());
            }
        };

        written_total += written_in_this_loop;

        // exit conditions below

        if written_in_this_loop == 0 {
            /*
            in case we read less than `buffer_window_size` in last `read`
            we need to shrink result because otherwise we'll return those
            un-read 0s
            */
            if written_total < result.len() {
                result.resize(written_total, 0);
            }
            return Ok(());
        }

        // TODO: It would be good to detect compression bombs, but
        // that would require access to the internal stream, which
        // would in turn require some tricky programming.  For now, we
        // use the maximum length here to prevent an attacker from
        // filling our RAM.
        if written_total > maxlen {
            result.resize(maxlen, 0);
            return Err(RequestError::ResponseTooLong(written_total));
        }
    }
}

/// Retire a directory circuit because of an error we've encountered on it.
fn retire_circ<R, E>(circ_mgr: &Arc<CircMgr<R>>, source_info: &SourceInfo, error: &E)
where
    R: Runtime,
    E: std::fmt::Display + ?Sized,
{
    let id = source_info.unique_circ_id();
    info!(
        "{}: Retiring circuit because of directory failure: {}",
        &id, &error
    );
    circ_mgr.retire_circ(id);
}

/// As AsyncBufReadExt::read_until, but stops after reading `max` bytes.
///
/// Note that this function might not actually read any byte of value
/// `byte`, since EOF might occur, or we might fill the buffer.
///
/// A return value of 0 indicates an end-of-file.
async fn read_until_limited<S>(
    stream: &mut S,
    byte: u8,
    max: usize,
    buf: &mut Vec<u8>,
) -> std::io::Result<usize>
where
    S: AsyncBufRead + Unpin,
{
    let mut n_added = 0;
    loop {
        let data = stream.fill_buf().await?;
        if data.is_empty() {
            // End-of-file has been reached.
            return Ok(n_added);
        }
        debug_assert!(n_added < max);
        let remaining_space = max - n_added;
        let (available, found_byte) = match memchr(byte, data) {
            Some(idx) => (idx + 1, true),
            None => (data.len(), false),
        };
        debug_assert!(available >= 1);
        let n_to_copy = std::cmp::min(remaining_space, available);
        buf.extend(&data[..n_to_copy]);
        stream.consume_unpin(n_to_copy);
        n_added += n_to_copy;
        if found_byte || n_added == max {
            return Ok(n_added);
        }
    }
}

/// Helper: Return a boxed decoder object that wraps the stream  $s.
macro_rules! decoder {
    ($dec:ident, $s:expr) => {{
        let mut decoder = $dec::new($s);
        decoder.multiple_members(true);
        Ok(Box::new(decoder))
    }};
}

/// Wrap `stream` in an appropriate type to undo the content encoding
/// as described in `encoding`.
fn get_decoder<'a, S: AsyncBufRead + Unpin + Send + 'a>(
    stream: S,
    encoding: Option<&str>,
) -> RequestResult<Box<dyn AsyncRead + Unpin + Send + 'a>> {
    match encoding {
        None | Some("identity") => Ok(Box::new(stream)),
        Some("deflate") => decoder!(ZlibDecoder, stream),
        #[cfg(feature = "xz")]
        Some("x-tor-lzma") => decoder!(XzDecoder, stream),
        #[cfg(feature = "zstd")]
        Some("x-zstd") => decoder!(ZstdDecoder, stream),
        Some(other) => Err(RequestError::ContentEncoding(other.into())),
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use tor_rtmock::{io::stream_pair, time::MockSleepProvider};

    use futures_await_test::async_test;

    #[async_test]
    async fn test_read_until_limited() -> RequestResult<()> {
        let mut out = Vec::new();
        let bytes = b"This line eventually ends\nthen comes another\n";

        // Case 1: find a whole line.
        let mut s = &bytes[..];
        let res = read_until_limited(&mut s, b'\n', 100, &mut out).await;
        assert_eq!(res?, 26);
        assert_eq!(&out[..], b"This line eventually ends\n");

        // Case 2: reach the limit.
        let mut s = &bytes[..];
        out.clear();
        let res = read_until_limited(&mut s, b'\n', 10, &mut out).await;
        assert_eq!(res?, 10);
        assert_eq!(&out[..], b"This line ");

        // Case 3: reach EOF.
        let mut s = &bytes[..];
        out.clear();
        let res = read_until_limited(&mut s, b'Z', 100, &mut out).await;
        assert_eq!(res?, 45);
        assert_eq!(&out[..], &bytes[..]);

        Ok(())
    }

    // Basic decompression wrapper.
    async fn decomp_basic(
        encoding: Option<&str>,
        data: &[u8],
        maxlen: usize,
    ) -> (RequestResult<()>, Vec<u8>) {
        // We don't need to do anything fancy here, since we aren't simulating
        // a timeout.
        let mock_time = MockSleepProvider::new(std::time::SystemTime::now());

        let mut output = Vec::new();
        let mut stream = match get_decoder(data, encoding) {
            Ok(s) => s,
            Err(e) => return (Err(e), output),
        };

        let r = read_and_decompress(&mock_time, &mut stream, maxlen, &mut output).await;

        (r, output)
    }

    #[async_test]
    async fn decompress_identity() -> RequestResult<()> {
        let mut text = Vec::new();
        for _ in 0..1000 {
            text.extend(b"This is a string with a nontrivial length that we'll use to make sure that the loop is executed more than once.");
        }

        let limit = 10 << 20;
        let (s, r) = decomp_basic(None, &text[..], limit).await;
        s?;
        assert_eq!(r, text);

        let (s, r) = decomp_basic(Some("identity"), &text[..], limit).await;
        s?;
        assert_eq!(r, text);

        // Try truncated result
        let limit = 100;
        let (s, r) = decomp_basic(Some("identity"), &text[..], limit).await;
        assert!(s.is_err());
        assert_eq!(r, &text[..100]);

        Ok(())
    }

    #[async_test]
    async fn decomp_zlib() -> RequestResult<()> {
        let compressed =
            hex::decode("789cf3cf4b5548cb2cce500829cf8730825253200ca79c52881c00e5970c88").unwrap();

        let limit = 10 << 20;
        let (s, r) = decomp_basic(Some("deflate"), &compressed, limit).await;
        s?;
        assert_eq!(r, b"One fish Two fish Red fish Blue fish");

        Ok(())
    }

    #[cfg(feature = "zstd")]
    #[async_test]
    async fn decomp_zstd() -> RequestResult<()> {
        let compressed = hex::decode("28b52ffd24250d0100c84f6e6520666973682054776f526564426c756520666973680a0200600c0e2509478352cb").unwrap();
        let limit = 10 << 20;
        let (s, r) = decomp_basic(Some("x-zstd"), &compressed, limit).await;
        s?;
        assert_eq!(r, b"One fish Two fish Red fish Blue fish\n");

        Ok(())
    }

    #[cfg(feature = "xz")]
    #[async_test]
    async fn decomp_xz2() -> RequestResult<()> {
        // Not so good at tiny files...
        let compressed = hex::decode("fd377a585a000004e6d6b446020021011c00000010cf58cce00024001d5d00279b88a202ca8612cfb3c19c87c34248a570451e4851d3323d34ab8000000000000901af64854c91f600013925d6ec06651fb6f37d010000000004595a").unwrap();
        let limit = 10 << 20;
        let (s, r) = decomp_basic(Some("x-tor-lzma"), &compressed, limit).await;
        s?;
        assert_eq!(r, b"One fish Two fish Red fish Blue fish\n");

        Ok(())
    }

    #[async_test]
    async fn decomp_unknown() {
        let compressed = hex::decode("28b52ffd24250d0100c84f6e6520666973682054776f526564426c756520666973680a0200600c0e2509478352cb").unwrap();
        let limit = 10 << 20;
        let (s, _r) = decomp_basic(Some("x-proprietary-rle"), &compressed, limit).await;

        assert!(matches!(s, Err(RequestError::ContentEncoding(_))));
    }

    #[async_test]
    async fn decomp_bad_data() {
        let compressed = b"This is not good zlib data";
        let limit = 10 << 20;
        let (s, _r) = decomp_basic(Some("deflate"), compressed, limit).await;

        // This should possibly be a different type in the future.
        assert!(matches!(s, Err(RequestError::IoError(_))));
    }

    #[async_test]
    async fn headers_ok() -> RequestResult<()> {
        let text = b"HTTP/1.0 200 OK\r\nDate: ignored\r\nContent-Encoding: Waffles\r\n\r\n";

        let mut s = &text[..];
        let h = read_headers(&mut s).await?;

        assert_eq!(h.status, Some(200));
        assert_eq!(h.encoding.as_deref(), Some("Waffles"));

        // now try truncated
        let mut s = &text[..15];
        let h = read_headers(&mut s).await;
        assert!(matches!(h, Err(RequestError::TruncatedHeaders)));

        // now try with no encoding.
        let text = b"HTTP/1.0 404 Not found\r\n\r\n";
        let mut s = &text[..];
        let h = read_headers(&mut s).await?;

        assert_eq!(h.status, Some(404));
        assert!(h.encoding.is_none());

        Ok(())
    }

    #[async_test]
    async fn headers_bogus() -> Result<()> {
        let text = b"HTTP/999.0 WHAT EVEN\r\n\r\n";
        let mut s = &text[..];
        let h = read_headers(&mut s).await;

        assert!(h.is_err());
        assert!(matches!(h, Err(RequestError::HttparseError(_))));
        Ok(())
    }

    /// Run a trivial download example with a response provided as a binary
    /// string.
    ///
    /// Return the directory response (if any) and the request as encoded (if
    /// any.)
    fn run_download_test<Req: request::Requestable>(
        req: Req,
        response: &[u8],
    ) -> (Result<DirResponse>, RequestResult<Vec<u8>>) {
        let (mut s1, s2) = stream_pair();
        let (mut s2_r, mut s2_w) = s2.split();

        tor_rtcompat::test_with_one_runtime!(|rt| async move {
            let rt2 = rt.clone();
            let (v1, v2, v3): (
                Result<DirResponse>,
                RequestResult<Vec<u8>>,
                RequestResult<()>,
            ) = futures::join!(
                async {
                    // Run the download function.
                    let r = download(&rt, &req, &mut s1, None).await;
                    s1.close().await.map_err(|error| Error::RequestFailed {
                        source: None,
                        error: error.into(),
                    })?;
                    r
                },
                async {
                    // Take the request from the client, and return it in "v2"
                    let mut v = Vec::new();
                    s2_r.read_to_end(&mut v).await?;
                    Ok(v)
                },
                async {
                    // Send back a response.
                    s2_w.write_all(response).await?;
                    // We wait a moment to give the other side time to notice it
                    // has data.
                    //
                    // (Tentative diagnosis: The `async-compress` crate seems to
                    // be behave differently depending on whether the "close"
                    // comes right after the incomplete data or whether it comes
                    // after a delay.  If there's a delay, it notices the
                    // truncated data and tells us about it. But when there's
                    // _no_delay, it treats the data as an error and doesn't
                    // tell our code.)

                    // TODO: sleeping in tests is not great.
                    rt2.sleep(Duration::from_millis(50)).await;
                    s2_w.close().await?;
                    Ok(())
                }
            );

            assert!(v3.is_ok());

            (v1, v2)
        })
    }

    #[test]
    fn test_download() -> RequestResult<()> {
        let req: request::MicrodescRequest = vec![[9; 32]].into_iter().collect();

        let (response, request) = run_download_test(
            req,
            b"HTTP/1.0 200 OK\r\n\r\nThis is where the descs would go.",
        );

        let request = request?;
        assert!(request[..].starts_with(
            b"GET /tor/micro/d/CQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQk.z HTTP/1.0\r\n"
        ));

        let response = response.unwrap();
        assert_eq!(response.status_code(), 200);
        assert!(!response.is_partial());
        assert!(response.error().is_none());
        assert!(response.source().is_none());
        let out_ref = response.output();
        assert_eq!(out_ref, b"This is where the descs would go.");
        let out = response.into_output();
        assert_eq!(&out, b"This is where the descs would go.");

        Ok(())
    }

    #[test]
    fn test_download_truncated() {
        // Request only one md, so "partial ok" will not be set.
        let req: request::MicrodescRequest = vec![[9; 32]].into_iter().collect();
        let mut response_text: Vec<u8> =
            (*b"HTTP/1.0 200 OK\r\nContent-Encoding: deflate\r\n\r\n").into();
        // "One fish two fish" as above twice, but truncated the second time
        response_text.extend(
            hex::decode("789cf3cf4b5548cb2cce500829cf8730825253200ca79c52881c00e5970c88").unwrap(),
        );
        response_text.extend(
            hex::decode("789cf3cf4b5548cb2cce500829cf8730825253200ca79c52881c00e5").unwrap(),
        );
        let (response, request) = run_download_test(req, &response_text);
        assert!(request.is_ok());
        assert!(response.is_err()); // The whole download should fail, since partial_ok wasn't set.

        // request two microdescs, so "partial_ok" will be set.
        let req: request::MicrodescRequest = vec![[9; 32]; 2].into_iter().collect();

        let (response, request) = run_download_test(req, &response_text);
        assert!(request.is_ok());

        let response = response.unwrap();
        assert_eq!(response.status_code(), 200);
        assert!(response.error().is_some());
        assert!(response.is_partial());
        assert!(response.output().len() < 37 * 2);
        assert!(response.output().starts_with(b"One fish"));
    }

    #[test]
    fn test_404() {
        let req: request::MicrodescRequest = vec![[9; 32]].into_iter().collect();
        let response_text = b"HTTP/1.0 418 I'm a teapot\r\n\r\n";
        let (response, _request) = run_download_test(req, response_text);

        assert_eq!(response.unwrap().status_code(), 418);
    }

    #[test]
    fn test_headers_truncated() {
        let req: request::MicrodescRequest = vec![[9; 32]].into_iter().collect();
        let response_text = b"HTTP/1.0 404 truncation happens here\r\n";
        let (response, _request) = run_download_test(req, response_text);

        assert!(matches!(
            response,
            Err(Error::RequestFailed {
                error: RequestError::TruncatedHeaders,
                ..
            })
        ));

        // Try a completely empty response.
        let req: request::MicrodescRequest = vec![[9; 32]].into_iter().collect();
        let response_text = b"";
        let (response, _request) = run_download_test(req, response_text);

        assert!(matches!(
            response,
            Err(Error::RequestFailed {
                error: RequestError::TruncatedHeaders,
                ..
            })
        ));
    }

    #[test]
    fn test_headers_too_long() {
        let req: request::MicrodescRequest = vec![[9; 32]].into_iter().collect();
        let mut response_text: Vec<u8> = (*b"HTTP/1.0 418 I'm a teapot\r\nX-Too-Many-As: ").into();
        response_text.resize(16384, b'A');
        let (response, _request) = run_download_test(req, &response_text);

        assert!(response.as_ref().unwrap_err().should_retire_circ());
        assert!(matches!(
            response,
            Err(Error::RequestFailed {
                error: RequestError::HttparseError(_),
                ..
            })
        ));
    }

    // TODO: test with bad utf-8
}
