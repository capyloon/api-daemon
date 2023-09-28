//! Traits for converting keys to and from OpenSSH format.
//
// TODO HSS (#902): OpenSSH keys can have passphrases. While the current implementation isn't able to
// handle such keys, we will eventually need to support them (this will be a breaking API change).

use ssh_key::private::KeypairData;
use ssh_key::Algorithm;

use crate::{EncodableKey, ErasedKey, KeyType, KeystoreError, Result};

use tor_error::{ErrorKind, HasKind};
use tor_llcrypto::pk::{ed25519, keymanip};
use zeroize::Zeroizing;

use std::path::PathBuf;
use std::sync::Arc;

/// An unparsed OpenSSH key.
///
/// Note: This is a wrapper around the contents of a file we think is an OpenSSH key. The inner
/// value is unchecked/unvalidated, and might not actually be a valid OpenSSH key.
///
/// The inner value is zeroed on drop.
pub(crate) struct UnparsedOpenSshKey {
    /// The contents of an OpenSSH key file.
    inner: Zeroizing<Vec<u8>>,
    /// The path of the file (for error reporting).
    path: PathBuf,
}

impl UnparsedOpenSshKey {
    /// Create a new [`UnparsedOpenSshKey`].
    ///
    /// The contents of `inner` are erased on drop.
    pub(crate) fn new(inner: Vec<u8>, path: PathBuf) -> Self {
        Self {
            inner: Zeroizing::new(inner),
            path,
        }
    }
}

/// SSH key algorithms.
//
// Note: this contains all the types supported by ssh_key, plus X25519.
#[derive(Copy, Clone, Debug, PartialEq, derive_more::Display)]
pub(crate) enum SshKeyAlgorithm {
    /// Digital Signature Algorithm
    Dsa,
    /// Elliptic Curve Digital Signature Algorithm
    Ecdsa,
    /// Ed25519
    Ed25519,
    /// X25519
    X25519,
    /// RSA
    Rsa,
    /// FIDO/U2F key with ECDSA/NIST-P256 + SHA-256
    SkEcdsaSha2NistP256,
    /// FIDO/U2F key with Ed25519
    SkEd25519,
    /// An unrecognized [`ssh_key::Algorithm`].
    Unknown(ssh_key::Algorithm),
}

impl From<Algorithm> for SshKeyAlgorithm {
    fn from(algo: Algorithm) -> SshKeyAlgorithm {
        match algo {
            Algorithm::Dsa => SshKeyAlgorithm::Dsa,
            Algorithm::Ecdsa { .. } => SshKeyAlgorithm::Ecdsa,
            Algorithm::Ed25519 => SshKeyAlgorithm::Ed25519,
            Algorithm::Rsa { .. } => SshKeyAlgorithm::Rsa,
            Algorithm::SkEcdsaSha2NistP256 => SshKeyAlgorithm::SkEcdsaSha2NistP256,
            Algorithm::SkEd25519 => SshKeyAlgorithm::SkEd25519,
            // Note: ssh_key::Algorithm is non_exhaustive, so we need this catch-all variant
            _ => SshKeyAlgorithm::Unknown(algo),
        }
    }
}

/// An error that occurred while processing an OpenSSH key.
#[derive(thiserror::Error, Debug, Clone)]
pub(crate) enum SshKeyError {
    /// Failed to parse an OpenSSH key
    #[error("Failed to parse OpenSSH with type {key_type:?}")]
    SshKeyParse {
        /// The path of the malformed key.
        path: PathBuf,
        /// The type of key we were trying to fetch.
        key_type: KeyType,
        /// The underlying error.
        #[source]
        err: Arc<ssh_key::Error>,
    },

    /// The OpenSSH key we retrieved is of the wrong type.
    #[error("Unexpected OpenSSH key type: wanted {wanted_key_algo}, found {found_key_algo}")]
    UnexpectedSshKeyType {
        /// The path of the malformed key.
        path: PathBuf,
        /// The algorithm we expected the key to use.
        wanted_key_algo: SshKeyAlgorithm,
        /// The algorithm of the key we got.
        found_key_algo: SshKeyAlgorithm,
    },
}

impl KeystoreError for SshKeyError {}

impl SshKeyError {
    /// A convenience method for boxing `self`.
    pub(crate) fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}

impl HasKind for SshKeyError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::KeystoreCorrupted
    }
}

/// A helper for reading Ed25519 OpenSSH private keys from disk.
fn read_ed25519_keypair(key_type: KeyType, key: UnparsedOpenSshKey) -> Result<ed25519::Keypair> {
    let sk =
        ssh_key::PrivateKey::from_openssh(&*key.inner).map_err(|e| SshKeyError::SshKeyParse {
            // TODO: rust thinks this clone is necessary because key.path is also used below (but
            // if we get to this point, we're going to return an error and never reach the other
            // error handling branches where we use key.path).
            path: key.path.clone(),
            key_type,
            err: e.into(),
        })?;

    // Build the expected key type (i.e. convert ssh_key key types to the key types
    // we're using internally).
    let key = match sk.key_data() {
        KeypairData::Ed25519(key) => {
            ed25519::Keypair::from_bytes(&key.to_bytes()).map_err(|_| {
                tor_error::internal!("failed to build ed25519 key out of ed25519 OpenSSH key")
            })?
        }
        _ => {
            return Err(SshKeyError::UnexpectedSshKeyType {
                path: key.path,
                wanted_key_algo: key_type.ssh_algorithm(),
                found_key_algo: sk.algorithm().into(),
            }
            .boxed());
        }
    };

    Ok(key)
}

impl KeyType {
    /// Get the algorithm of this key type.
    pub(crate) fn ssh_algorithm(&self) -> SshKeyAlgorithm {
        match self {
            KeyType::Ed25519Keypair => SshKeyAlgorithm::Ed25519,
            KeyType::X25519StaticSecret => SshKeyAlgorithm::X25519,
        }
    }

    /// Parse an OpenSSH key, convert the key material into a known key type, and return the
    /// type-erased value.
    ///
    /// The caller is expected to downcast the value returned to a concrete type.
    pub(crate) fn parse_ssh_format_erased(&self, key: UnparsedOpenSshKey) -> Result<ErasedKey> {
        // TODO HSS: perhaps this needs to be a method on EncodableKey instead?
        match self {
            KeyType::Ed25519Keypair => {
                read_ed25519_keypair(*self, key).map(|key| Box::new(key) as ErasedKey)
            }
            KeyType::X25519StaticSecret => {
                // NOTE: The ssh-key crate doesn't support storing curve25519 keys in OpenSSH
                // format. As a workaround, this type of key will be stored as ssh-ed25519 and
                // converted back to x25519 upon retrieval.
                let key = read_ed25519_keypair(KeyType::Ed25519Keypair, key)?;

                Ok(Box::new(keymanip::convert_ed25519_to_curve25519_private(
                    &key,
                )))
            }
        }
    }

    /// Encode an OpenSSH-formatted key.
    //
    // TODO HSS: remove "allow" and choose a better name for this function
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_ssh_format(&self, _key: &dyn EncodableKey) -> Result<String> {
        todo!() // TODO HSS
    }
}

#[cfg(test)]
mod tests {
    // @@ begin test lint list maintained by maint/add_warning @@
    #![allow(clippy::bool_assert_comparison)]
    #![allow(clippy::clone_on_copy)]
    #![allow(clippy::dbg_macro)]
    #![allow(clippy::print_stderr)]
    #![allow(clippy::print_stdout)]
    #![allow(clippy::single_char_pattern)]
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::unchecked_duration_subtraction)]
    //! <!-- @@ end test lint list maintained by maint/add_warning @@ -->
    use super::*;

    const OPENSSH_ED25519: &[u8] = include_bytes!("../../testdata/ed25519_openssh.private");
    const OPENSSH_ED25519_BAD: &[u8] = include_bytes!("../../testdata/ed25519_openssh_bad.private");
    const OPENSSH_DSA: &[u8] = include_bytes!("../../testdata/dsa_openssh.private");

    #[test]
    fn wrong_key_type() {
        let key_type = KeyType::Ed25519Keypair;
        let key = UnparsedOpenSshKey::new(OPENSSH_DSA.into(), PathBuf::from("/test/path"));
        let err = key_type.parse_ssh_format_erased(key).unwrap_err();

        assert_eq!(
            err.to_string(),
            format!(
                "Unexpected OpenSSH key type: wanted {}, found {}",
                SshKeyAlgorithm::Ed25519,
                SshKeyAlgorithm::Dsa
            )
        );
    }

    #[test]
    fn invalid_ed25519_key() {
        let key_type = KeyType::Ed25519Keypair;
        let key = UnparsedOpenSshKey::new(OPENSSH_ED25519_BAD.into(), PathBuf::from("/test/path"));
        let err = key_type.parse_ssh_format_erased(key).unwrap_err();

        assert_eq!(
            err.to_string(),
            "Failed to parse OpenSSH with type Ed25519Keypair"
        );
    }

    #[test]
    fn ed25519_key() {
        let key_type = KeyType::Ed25519Keypair;
        let key = UnparsedOpenSshKey::new(OPENSSH_ED25519.into(), PathBuf::from("/test/path"));
        let erased_key = key_type.parse_ssh_format_erased(key).unwrap();

        assert!(erased_key.downcast::<ed25519::Keypair>().is_ok());
    }
}
