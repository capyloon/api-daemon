//! Descriptions objects for different kinds of directory requests
//! that we can make.

use tor_llcrypto::pk::rsa::RsaIdentity;
use tor_netdoc::doc::authcert::AuthCertKeyIds;
use tor_netdoc::doc::microdesc::MdDigest;
use tor_netdoc::doc::netstatus::ConsensusFlavor;
#[cfg(feature = "routerdesc")]
use tor_netdoc::doc::routerdesc::RdDigest;
use tor_proto::circuit::ClientCirc;

/// Alias for a result with a `RequestError`.
type Result<T> = std::result::Result<T, crate::err::RequestError>;

use std::borrow::Cow;
use std::iter::FromIterator;
use std::time::{Duration, SystemTime};

use itertools::Itertools;

use crate::err::RequestError;

/// A request for an object that can be served over the Tor directory system.
pub trait Requestable {
    /// Build an [`http::Request`] from this Requestable, if
    /// it is well-formed.
    fn make_request(&self) -> Result<http::Request<()>>;

    /// Return true if partial downloads are potentially useful.  This
    /// is true for request types where we're going to be downloading
    /// multiple documents.
    fn partial_docs_ok(&self) -> bool;

    /// Return the maximum allowable response length we'll accept for this
    /// request.
    fn max_response_len(&self) -> usize {
        (16 * 1024 * 1024) - 1
    }

    /// Return an error if there is some problem with the provided circuit that
    /// would keep it from being used for this request.
    fn check_circuit(&self, circ: &ClientCirc) -> Result<()> {
        let _ = circ;
        Ok(())
    }
}

/// How much clock skew do we allow in the distance between the directory
/// cache's clock and our own?
///
///  If we find more skew than this, we end the
/// request early, on the theory that the directory will not tell us any
/// information we'd accept.
#[derive(Clone, Debug)]
struct SkewLimit {
    /// We refuse to proceed if the directory says we are more fast than this.
    ///
    /// (This is equivalent to deciding that, from our perspective, the
    /// directory is at least this slow.)
    max_fast: Duration,

    /// We refuse to proceed if the directory says that we are more slow than
    /// this.
    ///
    /// (This is equivalent to deciding that, from our perspective, the
    /// directory is at least this fast.)
    max_slow: Duration,
}

/// A Request for a consensus directory.
#[derive(Debug, Clone)]
pub struct ConsensusRequest {
    /// What flavor of consensus are we asking for?  Right now, only
    /// "microdesc" and "ns" are supported.
    flavor: ConsensusFlavor,
    /// A list of the authority identities that we believe in.  We tell the
    /// directory cache only to give us a consensus if it is signed by enough
    /// of these authorities.
    authority_ids: Vec<RsaIdentity>,
    /// The publication time of the most recent consensus we have.  Used to
    /// generate an If-Modified-Since header so that we don't get a document
    /// we already have.
    last_consensus_published: Option<SystemTime>,
    /// A set of SHA3-256 digests of the _signed portion_ of consensuses we have.
    /// Used to declare what diffs we would accept.
    ///
    /// (Currently we don't send this, since we can't handle diffs.)
    last_consensus_sha3_256: Vec<[u8; 32]>,
    /// If present, the largest amount of clock skew to allow between ourself and a directory cache.
    skew_limit: Option<SkewLimit>,
}

impl ConsensusRequest {
    /// Create a new request for a consensus directory document.
    pub fn new(flavor: ConsensusFlavor) -> Self {
        ConsensusRequest {
            flavor,
            authority_ids: Vec::new(),
            last_consensus_published: None,
            last_consensus_sha3_256: Vec::new(),
            skew_limit: None,
        }
    }

    /// Add `id` to the list of authorities that this request should
    /// say we believe in.
    pub fn push_authority_id(&mut self, id: RsaIdentity) {
        self.authority_ids.push(id);
    }

    /// Add `d` to the list of consensus digests this request should
    /// say we already have.
    pub fn push_old_consensus_digest(&mut self, d: [u8; 32]) {
        self.last_consensus_sha3_256.push(d);
    }

    /// Set the publication time we should say we have for our last
    /// consensus to `when`.
    pub fn set_last_consensus_date(&mut self, when: SystemTime) {
        self.last_consensus_published = Some(when);
    }

    /// Return a slice of the consensus digests that we're saying we
    /// already have.
    pub fn old_consensus_digests(&self) -> impl Iterator<Item = &[u8; 32]> {
        self.last_consensus_sha3_256.iter()
    }

    /// Return an iterator of the authority identities that this request
    /// is saying we believe in.
    pub fn authority_ids(&self) -> impl Iterator<Item = &RsaIdentity> {
        self.authority_ids.iter()
    }

    /// Return the date we're reporting for our most recent consensus.
    pub fn last_consensus_date(&self) -> Option<SystemTime> {
        self.last_consensus_published
    }

    /// Tell the directory client that we should abort the request early if the
    /// directory's clock skew exceeds certain limits.
    ///
    /// The `max_fast` parameter is the most fast that we're willing to be with
    /// respect to the directory (or in other words, the most slow that we're
    /// willing to let the directory be with respect to us).
    ///
    /// The `max_slow` parameter is the most _slow_ that we're willing to be with
    /// respect to the directory ((or in other words, the most slow that we're
    /// willing to let the directory be with respect to us).
    pub fn set_skew_limit(&mut self, max_fast: Duration, max_slow: Duration) {
        self.skew_limit = Some(SkewLimit { max_fast, max_slow });
    }
}

/// Convert a list of digests in some format to a string, for use in a request
///
/// The digests `DL` will be sorted, converted to strings with `EF`,
/// separated with `sep`, and returned as an fresh `String`.
///
/// If the digests list is empty, returns None instead.
//
// In principle this ought to be doable with much less allocating,
// starting with hex::encode etc.
fn digest_list_stringify<'d, D, DL, EF>(digests: DL, encode: EF, sep: &str) -> Option<String>
where
    DL: IntoIterator<Item = &'d D> + 'd,
    D: PartialOrd + Ord + 'd,
    EF: Fn(&'d D) -> String,
{
    let mut digests = digests.into_iter().collect_vec();
    if digests.is_empty() {
        return None;
    }
    digests.sort_unstable();
    let ids = digests.into_iter().map(encode).map(Cow::Owned);
    // name collision with unstable Iterator::intersperse
    // https://github.com/rust-lang/rust/issues/48919
    let ids = Itertools::intersperse(ids, Cow::Borrowed(sep)).collect::<String>();
    Some(ids)
}

impl Default for ConsensusRequest {
    fn default() -> Self {
        Self::new(ConsensusFlavor::Microdesc)
    }
}

impl Requestable for ConsensusRequest {
    fn make_request(&self) -> Result<http::Request<()>> {
        // Build the URL.
        let mut uri = "/tor/status-vote/current/consensus".to_string();
        match self.flavor {
            ConsensusFlavor::Ns => {}
            flav => {
                uri.push('-');
                uri.push_str(flav.name());
            }
        }
        let d_encode_hex = |id: &RsaIdentity| hex::encode(id.as_bytes());
        if let Some(ids) = digest_list_stringify(&self.authority_ids, d_encode_hex, "+") {
            // With authorities, "../consensus/<F1>+<F2>+<F3>.z"
            uri.push('/');
            uri.push_str(&ids);
        }
        // Without authorities, "../consensus-microdesc.z"
        uri.push_str(".z");

        let mut req = http::Request::builder().method("GET").uri(uri);
        req = add_common_headers(req);

        // Possibly, add an if-modified-since header.
        if let Some(when) = self.last_consensus_date() {
            req = req.header(
                http::header::IF_MODIFIED_SINCE,
                httpdate::fmt_http_date(when),
            );
        }

        // Possibly, add an X-Or-Diff-From-Consensus header.
        if let Some(ids) = digest_list_stringify(&self.last_consensus_sha3_256, hex::encode, ", ") {
            req = req.header("X-Or-Diff-From-Consensus", &ids);
        }

        Ok(req.body(())?)
    }

    fn partial_docs_ok(&self) -> bool {
        false
    }

    fn check_circuit(&self, circ: &ClientCirc) -> Result<()> {
        use tor_proto::ClockSkew::*;
        // This is the clock skew _according to the directory_.
        let skew = circ.channel().clock_skew();
        match (&self.skew_limit, &skew) {
            (Some(SkewLimit { max_slow, .. }), Slow(slow)) if slow > max_slow => {
                Err(RequestError::TooMuchClockSkew)
            }
            (Some(SkewLimit { max_fast, .. }), Fast(fast)) if fast > max_fast => {
                Err(RequestError::TooMuchClockSkew)
            }
            (_, _) => Ok(()),
        }
    }
}

/// A request for one or more authority certificates.
#[derive(Debug, Clone, Default)]
pub struct AuthCertRequest {
    /// The identity/signing keys of the certificates we want.
    ids: Vec<AuthCertKeyIds>,
}

impl AuthCertRequest {
    /// Create a new request, asking for no authority certificates.
    pub fn new() -> Self {
        AuthCertRequest::default()
    }

    /// Add `ids` to the list of certificates we're asking for.
    pub fn push(&mut self, ids: AuthCertKeyIds) {
        self.ids.push(ids);
    }

    /// Return a list of the keys that we're asking for.
    pub fn keys(&self) -> impl Iterator<Item = &AuthCertKeyIds> {
        self.ids.iter()
    }
}

impl Requestable for AuthCertRequest {
    fn make_request(&self) -> Result<http::Request<()>> {
        if self.ids.is_empty() {
            return Err(RequestError::EmptyRequest);
        }
        let mut ids = self.ids.clone();
        ids.sort_unstable();

        let ids: Vec<String> = ids
            .iter()
            .map(|id| {
                format!(
                    "{}-{}",
                    hex::encode(id.id_fingerprint.as_bytes()),
                    hex::encode(id.sk_fingerprint.as_bytes())
                )
            })
            .collect();

        let uri = format!("/tor/keys/fp-sk/{}.z", &ids.join("+"));

        let req = http::Request::builder().method("GET").uri(uri);
        let req = add_common_headers(req);

        Ok(req.body(())?)
    }

    fn partial_docs_ok(&self) -> bool {
        self.ids.len() > 1
    }

    fn max_response_len(&self) -> usize {
        // TODO: Pick a more principled number; I just made this one up.
        self.ids.len().saturating_mul(16 * 1024)
    }
}

impl FromIterator<AuthCertKeyIds> for AuthCertRequest {
    fn from_iter<I: IntoIterator<Item = AuthCertKeyIds>>(iter: I) -> Self {
        let mut req = Self::new();
        for i in iter {
            req.push(i);
        }
        req
    }
}

/// A request for one or more microdescriptors
#[derive(Debug, Clone, Default)]
pub struct MicrodescRequest {
    /// The SHA256 digests of the microdescriptors we want.
    digests: Vec<MdDigest>,
}

impl MicrodescRequest {
    /// Construct a request for no microdescriptors.
    pub fn new() -> Self {
        MicrodescRequest::default()
    }
    /// Add `d` to the list of microdescriptors we want to request.
    pub fn push(&mut self, d: MdDigest) {
        self.digests.push(d);
    }

    /// Return a list of the microdescriptor digests that we're asking for.
    pub fn digests(&self) -> impl Iterator<Item = &MdDigest> {
        self.digests.iter()
    }
}

impl Requestable for MicrodescRequest {
    fn make_request(&self) -> Result<http::Request<()>> {
        let d_encode_b64 = |d| base64::encode_config(d, base64::STANDARD_NO_PAD);
        let ids = digest_list_stringify(&self.digests, d_encode_b64, "-")
            .ok_or(RequestError::EmptyRequest)?;
        let uri = format!("/tor/micro/d/{}.z", &ids);
        let req = http::Request::builder().method("GET").uri(uri);

        let req = add_common_headers(req);

        Ok(req.body(())?)
    }

    fn partial_docs_ok(&self) -> bool {
        self.digests.len() > 1
    }

    fn max_response_len(&self) -> usize {
        // TODO: Pick a more principled number; I just made this one up.
        self.digests.len().saturating_mul(8 * 1024)
    }
}

impl FromIterator<MdDigest> for MicrodescRequest {
    fn from_iter<I: IntoIterator<Item = MdDigest>>(iter: I) -> Self {
        let mut req = Self::new();
        for i in iter {
            req.push(i);
        }
        req
    }
}

/// A request for one, many or all router descriptors.
#[derive(Debug, Clone, Default)]
#[cfg(feature = "routerdesc")]
pub struct RouterDescRequest {
    /// If this is set, we just ask for all the descriptors.
    // TODO: maybe this should be an enum, or maybe this case should
    // be a different type.
    all_descriptors: bool,
    /// A list of digests to download.
    digests: Vec<RdDigest>,
}

#[cfg(feature = "routerdesc")]
impl RouterDescRequest {
    /// Construct a request for all router descriptors.
    pub fn all() -> Self {
        RouterDescRequest {
            all_descriptors: true,
            digests: Vec::new(),
        }
    }
    /// Construct a new empty request.
    pub fn new() -> Self {
        RouterDescRequest::default()
    }
    /// Add `d` to the list of digests we want to request.
    pub fn push(&mut self, d: RdDigest) {
        if !self.all_descriptors {
            self.digests.push(d);
        }
    }

    /// Return an iterator over the descriptor digests that we're asking for.
    pub fn digests(&self) -> impl Iterator<Item = &RdDigest> {
        self.digests.iter()
    }
}

#[cfg(feature = "routerdesc")]
impl Requestable for RouterDescRequest {
    fn make_request(&self) -> Result<http::Request<()>> {
        let mut uri = "/tor/server/".to_string();

        if self.all_descriptors {
            uri.push_str("all");
        } else {
            uri.push_str("d/");
            let ids = digest_list_stringify(&self.digests, hex::encode, "+")
                .ok_or(RequestError::EmptyRequest)?;
            uri.push_str(&ids);
        }
        uri.push_str(".z");

        let req = http::Request::builder().method("GET").uri(uri);
        let req = add_common_headers(req);

        Ok(req.body(())?)
    }

    fn partial_docs_ok(&self) -> bool {
        self.digests.len() > 1 || self.all_descriptors
    }

    fn max_response_len(&self) -> usize {
        // TODO: Pick a more principled number; I just made these up.
        if self.all_descriptors {
            64 * 1024 * 1024 // big but not impossible
        } else {
            self.digests.len().saturating_mul(8 * 1024)
        }
    }
}

#[cfg(feature = "routerdesc")]
impl FromIterator<RdDigest> for RouterDescRequest {
    fn from_iter<I: IntoIterator<Item = RdDigest>>(iter: I) -> Self {
        let mut req = Self::new();
        for i in iter {
            req.push(i);
        }
        req
    }
}

/// List the encodings we accept
fn encodings() -> String {
    let mut encodings = "deflate, identity".to_string();
    #[cfg(feature = "xz")]
    {
        encodings += ", x-tor-lzma";
    }
    #[cfg(feature = "zstd")]
    {
        encodings += ", x-zstd";
    }

    encodings
}

/// Add commonly used headers to the HTTP request.
///
/// (Right now, this is only Accept-Encoding.)
fn add_common_headers(req: http::request::Builder) -> http::request::Builder {
    // TODO: gzip, brotli
    req.header(http::header::ACCEPT_ENCODING, encodings())
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn test_md_request() -> Result<()> {
        let d1 = b"This is a testing digest. it isn";
        let d2 = b"'t actually SHA-256.............";

        let mut req = MicrodescRequest::default();
        req.push(*d1);
        assert!(!req.partial_docs_ok());
        req.push(*d2);
        assert!(req.partial_docs_ok());
        assert_eq!(req.max_response_len(), 16 << 10);

        let req = crate::util::encode_request(&req.make_request()?);

        assert_eq!(req,
                   format!("GET /tor/micro/d/J3QgYWN0dWFsbHkgU0hBLTI1Ni4uLi4uLi4uLi4uLi4-VGhpcyBpcyBhIHRlc3RpbmcgZGlnZXN0LiBpdCBpc24.z HTTP/1.0\r\naccept-encoding: {}\r\n\r\n", encodings()));

        // Try it with FromIterator, and use some accessors.
        let req2: MicrodescRequest = vec![*d1, *d2].into_iter().collect();
        let ds: Vec<_> = req2.digests().collect();
        assert_eq!(ds, vec![d1, d2]);
        let req2 = crate::util::encode_request(&req2.make_request()?);
        assert_eq!(req, req2);

        Ok(())
    }

    #[test]
    fn test_cert_request() -> Result<()> {
        let d1 = b"This is a testing dn";
        let d2 = b"'t actually SHA-256.";
        let key1 = AuthCertKeyIds {
            id_fingerprint: (*d1).into(),
            sk_fingerprint: (*d2).into(),
        };

        let d3 = b"blah blah blah 1 2 3";
        let d4 = b"I like pizza from Na";
        let key2 = AuthCertKeyIds {
            id_fingerprint: (*d3).into(),
            sk_fingerprint: (*d4).into(),
        };

        let mut req = AuthCertRequest::default();
        req.push(key1);
        assert!(!req.partial_docs_ok());
        req.push(key2);
        assert!(req.partial_docs_ok());
        assert_eq!(req.max_response_len(), 32 << 10);

        let keys: Vec<_> = req.keys().collect();
        assert_eq!(keys, vec![&key1, &key2]);

        let req = crate::util::encode_request(&req.make_request()?);

        assert_eq!(req,
                   format!("GET /tor/keys/fp-sk/5468697320697320612074657374696e6720646e-27742061637475616c6c79205348412d3235362e+626c616820626c616820626c6168203120322033-49206c696b652070697a7a612066726f6d204e61.z HTTP/1.0\r\naccept-encoding: {}\r\n\r\n", encodings()));

        let req2: AuthCertRequest = vec![key1, key2].into_iter().collect();
        let req2 = crate::util::encode_request(&req2.make_request()?);
        assert_eq!(req, req2);

        Ok(())
    }

    #[test]
    fn test_consensus_request() -> Result<()> {
        let d1 = RsaIdentity::from_bytes(
            &hex::decode("03479E93EBF3FF2C58C1C9DBF2DE9DE9C2801B3E").unwrap(),
        )
        .unwrap();

        let d2 = b"blah blah blah 12 blah blah blah";
        let d3 = SystemTime::now();
        let mut req = ConsensusRequest::default();

        let when = httpdate::fmt_http_date(d3);

        req.push_authority_id(d1);
        req.push_old_consensus_digest(*d2);
        req.set_last_consensus_date(d3);
        assert!(!req.partial_docs_ok());
        assert_eq!(req.max_response_len(), (16 << 20) - 1);
        assert_eq!(req.old_consensus_digests().next(), Some(d2));
        assert_eq!(req.authority_ids().next(), Some(&d1));
        assert_eq!(req.last_consensus_date(), Some(d3));

        let req = crate::util::encode_request(&req.make_request()?);

        assert_eq!(req,
                   format!("GET /tor/status-vote/current/consensus-microdesc/03479e93ebf3ff2c58c1c9dbf2de9de9c2801b3e.z HTTP/1.0\r\naccept-encoding: {}\r\nif-modified-since: {}\r\nx-or-diff-from-consensus: 626c616820626c616820626c616820313220626c616820626c616820626c6168\r\n\r\n", encodings(), when));

        // Request without authorities
        let req = ConsensusRequest::default();
        let req = crate::util::encode_request(&req.make_request()?);
        assert_eq!(req,
                   format!("GET /tor/status-vote/current/consensus-microdesc.z HTTP/1.0\r\naccept-encoding: {}\r\n\r\n", encodings()));

        Ok(())
    }

    #[test]
    #[cfg(feature = "routerdesc")]
    fn test_rd_request_all() -> Result<()> {
        let req = RouterDescRequest::all();
        assert!(req.partial_docs_ok());
        assert_eq!(req.max_response_len(), 1 << 26);

        let req = crate::util::encode_request(&req.make_request()?);

        assert_eq!(
            req,
            format!(
                "GET /tor/server/all.z HTTP/1.0\r\naccept-encoding: {}\r\n\r\n",
                encodings()
            )
        );

        Ok(())
    }

    #[test]
    #[cfg(feature = "routerdesc")]
    fn test_rd_request() -> Result<()> {
        let d1 = b"at some point I got ";
        let d2 = b"of writing in hex...";

        let mut req = RouterDescRequest::default();
        req.push(*d1);
        assert!(!req.partial_docs_ok());
        req.push(*d2);
        assert!(req.partial_docs_ok());
        assert_eq!(req.max_response_len(), 16 << 10);

        let req = crate::util::encode_request(&req.make_request()?);

        assert_eq!(req,
                   format!("GET /tor/server/d/617420736f6d6520706f696e74204920676f7420+6f662077726974696e6720696e206865782e2e2e.z HTTP/1.0\r\naccept-encoding: {}\r\n\r\n", encodings()));

        // Try it with FromIterator, and use some accessors.
        let req2: RouterDescRequest = vec![*d1, *d2].into_iter().collect();
        let ds: Vec<_> = req2.digests().collect();
        assert_eq!(ds, vec![d1, d2]);
        let req2 = crate::util::encode_request(&req2.make_request()?);
        assert_eq!(req, req2);
        Ok(())
    }
}
