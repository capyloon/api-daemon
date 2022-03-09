//! Methods for storing and loading directory information from disk.
//!
//! We have code implemented for a flexible storage format based on sqlite.

// (There was once a read-only format based on the C tor implementation's
// storage: Search the git history for tor-dirmgr/src/storage/legacy.rs
// if you ever need to reinstate it.)

use tor_netdoc::doc::authcert::AuthCertKeyIds;
use tor_netdoc::doc::microdesc::MdDigest;
use tor_netdoc::doc::netstatus::ConsensusFlavor;

#[cfg(feature = "routerdesc")]
use tor_netdoc::doc::routerdesc::RdDigest;

use crate::docmeta::{AuthCertMeta, ConsensusMeta};
use crate::{Error, Result};
use std::cell::RefCell;
use std::collections::HashMap;
use std::time::SystemTime;
use std::{path::Path, str::Utf8Error};
use time::Duration;

pub(crate) mod sqlite;

pub(crate) use sqlite::SqliteStore;

/// Convenient Sized & dynamic [`Store`]
pub(crate) type DynStore = Box<dyn Store + Send>;

/// A document returned by a directory manager.
///
/// This document may be in memory, or may be mapped from a cache.  It is
/// not necessarily valid UTF-8.
pub struct DocumentText {
    /// The underlying InputString.  We only wrap this type to make it
    /// opaque to other crates, so they don't have to worry about the
    /// implementation details.
    s: InputString,
}

impl From<InputString> for DocumentText {
    fn from(s: InputString) -> DocumentText {
        DocumentText { s }
    }
}

impl AsRef<[u8]> for DocumentText {
    fn as_ref(&self) -> &[u8] {
        self.s.as_ref()
    }
}

impl DocumentText {
    /// Try to return a view of this document as a a string.
    pub(crate) fn as_str(&self) -> std::result::Result<&str, Utf8Error> {
        self.s.as_str_impl()
    }

    /// Create a new DocumentText holding the provided string.
    pub(crate) fn from_string(s: String) -> Self {
        DocumentText {
            s: InputString::Utf8(s),
        }
    }
}

/// An abstraction over a possible string that we've loaded or mapped from
/// a cache.
#[derive(Debug)]
pub(crate) enum InputString {
    /// A string that's been validated as UTF-8
    Utf8(String),
    /// A set of unvalidated bytes.
    UncheckedBytes {
        /// The underlying bytes
        bytes: Vec<u8>,
        /// Whether the bytes have been validated previously as UTF-8
        validated: RefCell<bool>,
    },
    #[cfg(feature = "mmap")]
    /// A set of memory-mapped bytes (not yet validated as UTF-8).
    MappedBytes {
        /// The underlying bytes
        bytes: memmap2::Mmap,
        /// Whether the bytes have been validated previously as UTF-8
        validated: RefCell<bool>,
    },
}

impl InputString {
    /// Return a view of this InputString as a &str, if it is valid UTF-8.
    pub(crate) fn as_str(&self) -> Result<&str> {
        self.as_str_impl()
            .map_err(|_| Error::CacheCorruption("Invalid UTF-8"))
    }

    /// Helper for [`Self::as_str()`], with unwrapped error type.
    fn as_str_impl(&self) -> std::result::Result<&str, Utf8Error> {
        // It is not necessary to re-check the UTF8 every time
        // this function is called so remember the result
        // we got with `validated`

        match self {
            InputString::Utf8(s) => Ok(&s[..]),
            InputString::UncheckedBytes { bytes, validated } => {
                if *validated.borrow() {
                    unsafe { Ok(std::str::from_utf8_unchecked(&bytes[..])) }
                } else {
                    let result = std::str::from_utf8(&bytes[..])?;
                    validated.replace(true);
                    Ok(result)
                }
            }
            #[cfg(feature = "mmap")]
            InputString::MappedBytes { bytes, validated } => {
                if *validated.borrow() {
                    unsafe { Ok(std::str::from_utf8_unchecked(&bytes[..])) }
                } else {
                    let result = std::str::from_utf8(&bytes[..])?;
                    validated.replace(true);
                    Ok(result)
                }
            }
        }
    }

    /// Construct a new InputString from a file on disk, trying to
    /// memory-map the file if possible.
    pub(crate) fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let f = std::fs::File::open(path)?;
        #[cfg(feature = "mmap")]
        {
            let mapping = unsafe {
                // I'd rather have a safe option, but that's not possible
                // with mmap, since other processes could in theory replace
                // the contents of the file while we're using it.
                memmap2::Mmap::map(&f)
            };
            if let Ok(bytes) = mapping {
                return Ok(InputString::MappedBytes {
                    bytes,
                    validated: RefCell::new(false),
                });
            }
        }
        use std::io::{BufReader, Read};
        let mut f = BufReader::new(f);
        let mut result = String::new();
        f.read_to_string(&mut result)?;
        Ok(InputString::Utf8(result))
    }
}

impl AsRef<[u8]> for InputString {
    fn as_ref(&self) -> &[u8] {
        match self {
            InputString::Utf8(s) => s.as_ref(),
            InputString::UncheckedBytes { bytes, .. } => &bytes[..],
            #[cfg(feature = "mmap")]
            InputString::MappedBytes { bytes, .. } => &bytes[..],
        }
    }
}

impl From<String> for InputString {
    fn from(s: String) -> InputString {
        InputString::Utf8(s)
    }
}

impl From<Vec<u8>> for InputString {
    fn from(bytes: Vec<u8>) -> InputString {
        InputString::UncheckedBytes {
            bytes,
            validated: RefCell::new(false),
        }
    }
}

/// Configuration of expiration of each element of a [`Store`].
pub(crate) struct ExpirationConfig {
    /// How long to keep expired router descriptors.
    pub(super) router_descs: Duration,
    /// How long to keep expired microdescriptors descriptors.
    pub(super) microdescs: Duration,
    /// How long to keep expired authority certificate.
    pub(super) authcerts: Duration,
    /// How long to keep expired consensus.
    pub(super) consensuses: Duration,
}

/// Configuration of expiration shared between [`Store`] implementations.
pub(crate) const EXPIRATION_DEFAULTS: ExpirationConfig = {
    ExpirationConfig {
        // TODO: Choose a more realistic time.
        router_descs: Duration::days(3 * 30),
        // TODO: Choose a more realistic time.
        microdescs: Duration::days(3 * 30),
        authcerts: Duration::ZERO,
        consensuses: Duration::days(2),
    }
};

/// Representation of a storage.
///
/// When creating an instance of this [`Store`], it should try to grab the lock during
/// initialization (`is_readonly() iff some other implementation grabbed it`).
pub(crate) trait Store {
    /// Return true if this [`Store`] is opened in read-only mode.
    fn is_readonly(&self) -> bool;
    /// Try to upgrade from a read-only connection to a read-write connection.
    ///
    /// Return true on success; false if another process had the lock.
    fn upgrade_to_readwrite(&mut self) -> Result<bool>;

    /// Delete all completely-expired objects from the database.
    ///
    /// This is pretty conservative, and only removes things that are
    /// definitely past their good-by date.
    fn expire_all(&mut self, expiration: &ExpirationConfig) -> Result<()>;

    /// Load the latest consensus from disk.
    ///
    /// If `pending` is given, we will only return a consensus with
    /// the given "pending" status.  (A pending consensus doesn't have
    /// enough descriptors yet.)  If `pending_ok` is None, we'll
    /// return a consensus with any pending status.
    fn latest_consensus(
        &self,
        flavor: ConsensusFlavor,
        pending: Option<bool>,
    ) -> Result<Option<InputString>>;
    /// Return the information about the latest non-pending consensus,
    /// including its valid-after time and digest.
    fn latest_consensus_meta(&self, flavor: ConsensusFlavor) -> Result<Option<ConsensusMeta>>;
    /// Try to read the consensus corresponding to the provided metadata object.
    fn consensus_by_meta(&self, cmeta: &ConsensusMeta) -> Result<InputString>;
    /// Try to read the consensus whose SHA3-256 digests is the provided
    /// value, and its metadata.
    fn consensus_by_sha3_digest_of_signed_part(
        &self,
        d: &[u8; 32],
    ) -> Result<Option<(InputString, ConsensusMeta)>>;
    /// Write a consensus to disk.
    fn store_consensus(
        &mut self,
        cmeta: &ConsensusMeta,
        flavor: ConsensusFlavor,
        pending: bool,
        contents: &str,
    ) -> Result<()>;
    /// Mark the consensus generated from `cmeta` as no longer pending.
    fn mark_consensus_usable(&mut self, cmeta: &ConsensusMeta) -> Result<()>;
    /// Remove the consensus generated from `cmeta`.
    fn delete_consensus(&mut self, cmeta: &ConsensusMeta) -> Result<()>;

    /// Read all of the specified authority certs from the cache.
    fn authcerts(&self, certs: &[AuthCertKeyIds]) -> Result<HashMap<AuthCertKeyIds, String>>;
    /// Save a list of authority certificates to the cache.
    fn store_authcerts(&mut self, certs: &[(AuthCertMeta, &str)]) -> Result<()>;

    /// Read all the microdescriptors listed in `input` from the cache.
    fn microdescs(&self, digests: &[MdDigest]) -> Result<HashMap<MdDigest, String>>;
    /// Store every microdescriptor in `input` into the cache, and say that
    /// it was last listed at `when`.
    fn store_microdescs(&mut self, digests: &[(&str, &MdDigest)], when: SystemTime) -> Result<()>;
    /// Update the `last-listed` time of every microdescriptor in
    /// `input` to `when` or later.
    fn update_microdescs_listed(&mut self, digests: &[MdDigest], when: SystemTime) -> Result<()>;

    /// Read all the microdescriptors listed in `input` from the cache.
    ///
    /// Only available when the `routerdesc` feature is present.
    #[cfg(feature = "routerdesc")]
    fn routerdescs(&self, digests: &[RdDigest]) -> Result<HashMap<RdDigest, String>>;
    /// Store every router descriptors in `input` into the cache.
    #[cfg(feature = "routerdesc")]
    #[allow(unused)]
    fn store_routerdescs(&mut self, digests: &[(&str, SystemTime, &RdDigest)]) -> Result<()>;
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn strings() {
        let s: InputString = "Hello world".to_string().into();
        assert_eq!(s.as_ref(), b"Hello world");
        assert_eq!(s.as_str().unwrap(), "Hello world");
        assert_eq!(s.as_str().unwrap(), "Hello world");

        let s: InputString = b"Hello world".to_vec().into();
        assert_eq!(s.as_ref(), b"Hello world");
        assert_eq!(s.as_str().unwrap(), "Hello world");
        assert_eq!(s.as_str().unwrap(), "Hello world");

        // bad utf-8
        let s: InputString = b"Hello \xff world".to_vec().into();
        assert_eq!(s.as_ref(), b"Hello \xff world");
        assert!(s.as_str().is_err());
    }

    #[test]
    fn files() {
        let td = tempdir().unwrap();

        let absent = td.path().join("absent");
        let s = InputString::load(&absent);
        assert!(s.is_err());

        let goodstr = td.path().join("goodstr");
        std::fs::write(&goodstr, "This is a reasonable file.\n").unwrap();
        let s = InputString::load(&goodstr);
        let s = s.unwrap();
        assert_eq!(s.as_str().unwrap(), "This is a reasonable file.\n");
        assert_eq!(s.as_str().unwrap(), "This is a reasonable file.\n");
        assert_eq!(s.as_ref(), b"This is a reasonable file.\n");

        let badutf8 = td.path().join("badutf8");
        std::fs::write(&badutf8, b"Not good \xff UTF-8.\n").unwrap();
        let s = InputString::load(&badutf8);
        assert!(s.is_err() || s.unwrap().as_str().is_err());
    }

    #[test]
    fn doctext() {
        let s: InputString = "Hello universe".to_string().into();
        let dt: DocumentText = s.into();
        assert_eq!(dt.as_ref(), b"Hello universe");
        assert_eq!(dt.as_str(), Ok("Hello universe"));
        assert_eq!(dt.as_str(), Ok("Hello universe"));

        let s: InputString = b"Hello \xff universe".to_vec().into();
        let dt: DocumentText = s.into();
        assert_eq!(dt.as_ref(), b"Hello \xff universe");
        assert!(dt.as_str().is_err());
    }
}
