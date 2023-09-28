//! Re-exporting Ed25519 implementations, and related utilities.
//!
//! Here we re-export types from [`ed25519_dalek`] that implement the
//! Ed25519 signature algorithm.  (TODO: Eventually, this module
//! should probably be replaced with a wrapper that uses the ed25519
//! trait and the Signature trait.)
//!
//! We additionally provide an `Ed25519Identity` type to represent the
//! unvalidated Ed25519 "identity keys" that we use throughout the Tor
//! protocol to uniquely identify a relay.

use base64ct::{Base64Unpadded, Encoding as _};
use std::fmt::{self, Debug, Display, Formatter};
use subtle::{Choice, ConstantTimeEq};

pub use ed25519_dalek::{ExpandedSecretKey, Keypair, PublicKey, SecretKey, Signature, Signer};

use crate::util::ct::CtByteArray;

/// The length of an ED25519 identity, in bytes.
pub const ED25519_ID_LEN: usize = 32;

/// The length of an ED25519 signature, in bytes.
pub const ED25519_SIGNATURE_LEN: usize = 64;

/// A variant of [`Keypair`] containing an [`ExpandedSecretKey`].
#[allow(clippy::exhaustive_structs)]
pub struct ExpandedKeypair {
    /// The secret part of the key.
    pub secret: ExpandedSecretKey,
    /// The public part of this key.
    pub public: PublicKey,
}

impl ExpandedKeypair {
    /// Compute a signature over a message using this keypair.
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.secret.sign(message, &self.public)
    }
}

impl<'a> From<&'a Keypair> for ExpandedKeypair {
    fn from(kp: &'a Keypair) -> ExpandedKeypair {
        ExpandedKeypair {
            secret: (&kp.secret).into(),
            public: kp.public,
        }
    }
}

/// An unchecked, unvalidated Ed25519 key.
///
/// This key is an "identity" in the sense that it identifies (up to) one
/// Ed25519 key.  It may also represent the identity for a particular entity,
/// such as a relay or an onion service.
///
/// This type is distinct from an Ed25519 [`PublicKey`] for several reasons:
///  * We're storing it in a compact format, whereas the public key
///    implementation might want an expanded form for more efficient key
///    validation.
///  * This type hasn't checked whether the bytes here actually _are_ a valid
///    Ed25519 public key.
#[derive(Clone, Copy, Hash, PartialOrd, Ord, Eq, PartialEq)]
pub struct Ed25519Identity {
    /// A raw unchecked Ed25519 public key.
    id: CtByteArray<ED25519_ID_LEN>,
}

impl Ed25519Identity {
    /// Construct a new Ed25519 identity from a 32-byte sequence.
    ///
    /// This might or might not actually be a valid Ed25519 public key.
    ///
    /// ```
    /// use tor_llcrypto::pk::ed25519::{Ed25519Identity, PublicKey};
    ///
    /// let bytes = b"klsadjfkladsfjklsdafkljasdfsdsd!";
    /// let id = Ed25519Identity::new(*bytes);
    /// let pk: Result<PublicKey,_> = (&id).try_into();
    /// assert!(pk.is_ok());
    ///
    /// let bytes = b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    /// let id = Ed25519Identity::new(*bytes);
    /// let pk: Result<PublicKey,_> = (&id).try_into();
    /// assert!(pk.is_err());
    /// ```
    pub fn new(id: [u8; 32]) -> Self {
        Ed25519Identity { id: id.into() }
    }
    /// If `id` is of the correct length, wrap it in an Ed25519Identity.
    pub fn from_bytes(id: &[u8]) -> Option<Self> {
        Some(Ed25519Identity::new(id.try_into().ok()?))
    }
    /// Return a reference to the bytes in this key.
    pub fn as_bytes(&self) -> &[u8] {
        &self.id.as_ref()[..]
    }
}

impl From<[u8; ED25519_ID_LEN]> for Ed25519Identity {
    fn from(id: [u8; ED25519_ID_LEN]) -> Self {
        Ed25519Identity::new(id)
    }
}

impl From<Ed25519Identity> for [u8; ED25519_ID_LEN] {
    fn from(value: Ed25519Identity) -> Self {
        value.id.into()
    }
}

impl From<PublicKey> for Ed25519Identity {
    fn from(pk: PublicKey) -> Self {
        (&pk).into()
    }
}

impl From<&PublicKey> for Ed25519Identity {
    fn from(pk: &PublicKey) -> Self {
        // This unwrap is safe because the public key is always 32 bytes
        // long.
        Ed25519Identity::from_bytes(pk.as_bytes()).expect("Ed25519 public key had wrong length?")
    }
}

impl TryFrom<&Ed25519Identity> for PublicKey {
    type Error = ed25519_dalek::SignatureError;
    fn try_from(id: &Ed25519Identity) -> Result<PublicKey, Self::Error> {
        PublicKey::from_bytes(&id.id.as_ref()[..])
    }
}

impl TryFrom<Ed25519Identity> for PublicKey {
    type Error = ed25519_dalek::SignatureError;
    fn try_from(id: Ed25519Identity) -> Result<PublicKey, Self::Error> {
        (&id).try_into()
    }
}

impl ConstantTimeEq for Ed25519Identity {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.id.ct_eq(&other.id)
    }
}

impl Display for Ed25519Identity {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Base64Unpadded::encode_string(self.id.as_ref()))
    }
}

impl Debug for Ed25519Identity {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Ed25519Identity {{ {} }}", self)
    }
}

impl safelog::Redactable for Ed25519Identity {
    /// Warning: This displays 12 bits of the ed25519 identity, which is
    /// enough to narrow down a public relay by a great deal.
    fn display_redacted(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}…",
            &Base64Unpadded::encode_string(self.id.as_ref())[..2]
        )
    }

    fn debug_redacted(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ed25519Identity {{ {} }}", self.redacted())
    }
}

impl serde::Serialize for Ed25519Identity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&Base64Unpadded::encode_string(self.id.as_ref()))
        } else {
            serializer.serialize_bytes(&self.id.as_ref()[..])
        }
    }
}

impl<'de> serde::Deserialize<'de> for Ed25519Identity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            /// Helper for deserialization
            struct EdIdentityVisitor;
            impl<'de> serde::de::Visitor<'de> for EdIdentityVisitor {
                type Value = Ed25519Identity;
                fn expecting(&self, fmt: &mut std::fmt::Formatter<'_>) -> fmt::Result {
                    fmt.write_str("base64-encoded Ed25519 public key")
                }
                fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    let bytes = Base64Unpadded::decode_vec(s).map_err(E::custom)?;
                    Ed25519Identity::from_bytes(&bytes)
                        .ok_or_else(|| E::custom("wrong length for Ed25519 public key"))
                }
            }

            deserializer.deserialize_str(EdIdentityVisitor)
        } else {
            /// Helper for deserialization
            struct EdIdentityVisitor;
            impl<'de> serde::de::Visitor<'de> for EdIdentityVisitor {
                type Value = Ed25519Identity;
                fn expecting(&self, fmt: &mut std::fmt::Formatter<'_>) -> fmt::Result {
                    fmt.write_str("ed25519 public key")
                }
                fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    Ed25519Identity::from_bytes(bytes)
                        .ok_or_else(|| E::custom("wrong length for ed25519 public key"))
                }
            }
            deserializer.deserialize_bytes(EdIdentityVisitor)
        }
    }
}

/// An ed25519 signature, plus the document that it signs and its
/// public key.
#[derive(Clone, Debug)]
pub struct ValidatableEd25519Signature {
    /// The key that allegedly produced the signature
    key: PublicKey,
    /// The alleged signature
    sig: Signature,
    /// The entire body of text that is allegedly signed here.
    ///
    /// TODO: It's not so good to have this included here; it
    /// would be better to have a patch to ed25519_dalek to allow
    /// us to pre-hash the signed thing, and just store a digest.
    /// We can't use that with the 'prehash' variant of ed25519,
    /// since that has different constants.
    entire_text_of_signed_thing: Vec<u8>,
}

impl ValidatableEd25519Signature {
    /// Create a new ValidatableEd25519Signature
    pub fn new(key: PublicKey, sig: Signature, text: &[u8]) -> Self {
        ValidatableEd25519Signature {
            key,
            sig,
            entire_text_of_signed_thing: text.into(),
        }
    }

    /// View the interior of this signature object.
    pub(crate) fn as_parts(&self) -> (&PublicKey, &Signature, &[u8]) {
        (&self.key, &self.sig, &self.entire_text_of_signed_thing[..])
    }

    /// Return a reference to the underlying Signature.
    pub fn signature(&self) -> &Signature {
        &self.sig
    }
}

impl super::ValidatableSignature for ValidatableEd25519Signature {
    fn is_valid(&self) -> bool {
        use signature::Verifier;
        self.key
            .verify(&self.entire_text_of_signed_thing[..], &self.sig)
            .is_ok()
    }

    fn as_ed25519(&self) -> Option<&ValidatableEd25519Signature> {
        Some(self)
    }
}

/// Perform a batch verification operation on the provided signatures
///
/// Return `true` if _every_ signature is valid; otherwise return `false`.
///
/// Note that the mathematics for batch validation are slightly
/// different than those for normal one-signature validation.  Because
/// of this, it is possible for an ostensible signature that passes
/// one validation algorithm might fail the other.  (Well-formed
/// signatures generated by a correct Ed25519 implementation will
/// always pass both kinds of validation, and an attacker should not
/// be able to forge a signature that passes either kind.)
pub fn validate_batch(sigs: &[&ValidatableEd25519Signature]) -> bool {
    use crate::pk::ValidatableSignature;
    if sigs.is_empty() {
        // ed25519_dalek has nonzero cost for a batch-verification of
        // zero sigs.
        true
    } else if sigs.len() == 1 {
        // Validating one signature in the traditional way is faster.
        sigs[0].is_valid()
    } else {
        let mut ed_msgs = Vec::new();
        let mut ed_sigs = Vec::new();
        let mut ed_pks = Vec::new();
        for ed_sig in sigs {
            let (pk, sig, msg) = ed_sig.as_parts();
            ed_sigs.push(*sig);
            ed_pks.push(*pk);
            ed_msgs.push(msg);
        }
        ed25519_dalek::verify_batch(&ed_msgs[..], &ed_sigs[..], &ed_pks[..]).is_ok()
    }
}

/// An object that has an Ed25519 [`PublicKey`].
pub trait Ed25519PublicKey {
    /// Get the Ed25519 [`PublicKey`].
    fn public_key(&self) -> &PublicKey;
}

impl Ed25519PublicKey for Keypair {
    fn public_key(&self) -> &PublicKey {
        &self.public
    }
}
