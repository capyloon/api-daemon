//! Re-exporting RSA implementations.
//!
//! This module can currently handle public keys and signature
//! verification used in the Tor directory protocol and
//! similar places.
//!
//! Currently, that means validating PKCSv1 signatures, and encoding
//! and decoding RSA public keys from DER.
//!
//! # Limitations:
//!
//! Currently missing are support for signing and RSA-OEAP.  In Tor,
//! RSA signing is only needed for relays and authorities, and
//! RSA-OAEP padding is only needed for the (obsolete) TAP protocol.
//!
//! This module should expose RustCrypto trait-based wrappers,
//! but the [`rsa`] crate didn't support them as of initial writing.
use arrayref::array_ref;
use rsa::pkcs1::{DecodeRsaPrivateKey, DecodeRsaPublicKey};
use std::fmt;
use subtle::{Choice, ConstantTimeEq};

/// How many bytes are in an "RSA ID"?  (This is a legacy tor
/// concept, and refers to identifying a relay by a SHA1 digest
/// of its RSA public identity key.)
pub const RSA_ID_LEN: usize = 20;

/// An identifier for a Tor relay, based on its legacy RSA identity
/// key.  These are used all over the Tor protocol.
///
/// Note that for modern purposes, you should almost always identify a
/// relay by its [`Ed25519Identity`](crate::pk::ed25519::Ed25519Identity)
/// instead of by this kind of identity key.
#[derive(Clone, Copy, Hash, Ord, PartialOrd)]
#[allow(clippy::derive_hash_xor_eq)]
pub struct RsaIdentity {
    /// SHA1 digest of a DER encoded public key.
    id: [u8; RSA_ID_LEN],
}

impl ConstantTimeEq for RsaIdentity {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.id.ct_eq(&other.id)
    }
}

impl PartialEq<RsaIdentity> for RsaIdentity {
    fn eq(&self, rhs: &RsaIdentity) -> bool {
        self.ct_eq(rhs).unwrap_u8() == 1
    }
}

impl Eq for RsaIdentity {}

impl fmt::Display for RsaIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${}", hex::encode(&self.id[..]))
    }
}
impl fmt::Debug for RsaIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RsaIdentity {{ {} }}", self)
    }
}

impl serde::Serialize for RsaIdentity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&hex::encode(&self.id[..]))
        } else {
            serializer.serialize_bytes(&self.id[..])
        }
    }
}

impl<'de> serde::Deserialize<'de> for RsaIdentity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            /// Deserialization helper
            struct RsaIdentityVisitor;
            impl<'de> serde::de::Visitor<'de> for RsaIdentityVisitor {
                type Value = RsaIdentity;
                fn expecting(&self, fmt: &mut std::fmt::Formatter<'_>) -> fmt::Result {
                    fmt.write_str("hex-encoded RSA identity")
                }
                fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    RsaIdentity::from_hex(s)
                        .ok_or_else(|| E::custom("wrong encoding for RSA identity"))
                }
            }

            deserializer.deserialize_str(RsaIdentityVisitor)
        } else {
            /// Deserialization helper
            struct RsaIdentityVisitor;
            impl<'de> serde::de::Visitor<'de> for RsaIdentityVisitor {
                type Value = RsaIdentity;
                fn expecting(&self, fmt: &mut std::fmt::Formatter<'_>) -> fmt::Result {
                    fmt.write_str("RSA identity")
                }
                fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    RsaIdentity::from_bytes(bytes)
                        .ok_or_else(|| E::custom("wrong length for RSA identity"))
                }
            }
            deserializer.deserialize_bytes(RsaIdentityVisitor)
        }
    }
}

impl RsaIdentity {
    /// Expose an RsaIdentity as a slice of bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.id[..]
    }
    /// Construct an RsaIdentity from a slice of bytes.
    ///
    /// Returns None if the input is not of the correct length.
    ///
    /// ```
    /// use tor_llcrypto::pk::rsa::RsaIdentity;
    ///
    /// let bytes = b"xyzzyxyzzyxyzzyxyzzy";
    /// let id = RsaIdentity::from_bytes(bytes);
    /// assert_eq!(id.unwrap().as_bytes(), bytes);
    ///
    /// let truncated = b"xyzzy";
    /// let id = RsaIdentity::from_bytes(truncated);
    /// assert_eq!(id, None);
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() == RSA_ID_LEN {
            Some(RsaIdentity {
                id: *array_ref![bytes, 0, RSA_ID_LEN],
            })
        } else {
            None
        }
    }
    /// Decode an `RsaIdentity` from a hexadecimal string.
    ///
    /// The string must have no spaces, or any extra characters.
    pub fn from_hex(s: &str) -> Option<Self> {
        let mut array = [0_u8; 20];
        match hex::decode_to_slice(s, &mut array) {
            Err(_) => None,
            Ok(()) => Some(RsaIdentity::from(array)),
        }
    }
}

impl From<[u8; 20]> for RsaIdentity {
    fn from(id: [u8; 20]) -> RsaIdentity {
        RsaIdentity { id }
    }
}

/// An RSA public key.
///
/// This implementation is a simple wrapper so that we can define new
/// methods and traits on the type.
#[derive(Clone, Debug)]
pub struct PublicKey(rsa::RsaPublicKey);

/// An RSA private key.
///
/// This is not so useful at present, since Arti currently only has
/// client support, and Tor clients never actually need RSA private
/// keys.
pub struct PrivateKey(rsa::RsaPrivateKey);

impl PrivateKey {
    /// Return the public component of this key.
    pub fn to_public_key(&self) -> PublicKey {
        PublicKey(self.0.to_public_key())
    }
    /// Construct a PrivateKey from DER pkcs1 encoding.
    pub fn from_der(der: &[u8]) -> Option<Self> {
        Some(PrivateKey(rsa::RsaPrivateKey::from_pkcs1_der(der).ok()?))
    }
    // ....
}
impl PublicKey {
    /// Return true iff the exponent for this key is the same
    /// number as 'e'.
    pub fn exponent_is(&self, e: u32) -> bool {
        use rsa::PublicKeyParts;
        *self.0.e() == rsa::BigUint::new(vec![e])
    }
    /// Return the number of bits in the modulus for this key.
    pub fn bits(&self) -> usize {
        use rsa::PublicKeyParts;
        self.0.n().bits()
    }
    /// Try to check a signature (as used in Tor.)  The signed hash
    /// should be in 'hashed', and the alleged signature in 'sig'.
    ///
    /// Tor uses RSA-PKCSv1 signatures, with hash algorithm OIDs
    /// omitted.
    pub fn verify(&self, hashed: &[u8], sig: &[u8]) -> Result<(), signature::Error> {
        use rsa::PublicKey;
        let padding = rsa::PaddingScheme::new_pkcs1v15_sign(None);
        self.0
            .verify(padding, hashed, sig)
            .map_err(|_| signature::Error::new())
    }
    /// Decode an alleged DER byte string into a PublicKey.
    ///
    /// Return None  if the DER string does not have a valid PublicKey.
    ///
    /// (This function expects an RsaPublicKey, as used by Tor.  It
    /// does not expect or accept a PublicKeyInfo.)
    pub fn from_der(der: &[u8]) -> Option<Self> {
        Some(PublicKey(rsa::RsaPublicKey::from_pkcs1_der(der).ok()?))
    }
    /// Encode this public key into the DER format as used by Tor.
    ///
    /// The result is an RsaPublicKey, not a PublicKeyInfo.
    pub fn to_der(&self) -> Vec<u8> {
        // There seem to be version issues with these two
        // versions of bigint: yuck!
        use rsa::BigUint; // not the same as the one in simple_asn1.
        use rsa::PublicKeyParts;
        use simple_asn1::{ASN1Block, BigInt};
        /// Helper: convert a BigUInt to signed asn1.
        fn to_asn1_int(x: &BigUint) -> ASN1Block {
            // We stick a "0" on the front so that we can used
            // from_signed_bytes_be.  The 0 guarantees that we'll
            // have a positive value.
            let mut bytes = vec![0];
            bytes.extend(x.to_bytes_be());
            // We use from_signed_bytes_be() here because simple_asn1
            // exposes BigInt but not Sign, so we can't call
            // its version of from_signed_bytes().
            let bigint = BigInt::from_signed_bytes_be(&bytes);
            ASN1Block::Integer(0, bigint)
        }

        let asn1 = ASN1Block::Sequence(0, vec![to_asn1_int(self.0.n()), to_asn1_int(self.0.e())]);
        simple_asn1::to_der(&asn1).expect("RSA key not encodable as DER")
    }

    /// Compute the RsaIdentity for this public key.
    pub fn to_rsa_identity(&self) -> RsaIdentity {
        use crate::d::Sha1;
        use digest::Digest;
        let id = Sha1::digest(&self.to_der()).into();
        RsaIdentity { id }
    }
}

/// An RSA signature plus all the information needed to validate it.
pub struct ValidatableRsaSignature {
    /// The key that allegedly signed this signature
    key: PublicKey,
    /// The signature in question
    sig: Vec<u8>,
    /// The value we expect to find that the signature is a signature of.
    expected_hash: Vec<u8>,
}

impl ValidatableRsaSignature {
    /// Construct a new ValidatableRsaSignature.
    pub fn new(key: &PublicKey, sig: &[u8], expected_hash: &[u8]) -> Self {
        ValidatableRsaSignature {
            key: key.clone(),
            sig: sig.into(),
            expected_hash: expected_hash.into(),
        }
    }
}

impl super::ValidatableSignature for ValidatableRsaSignature {
    fn is_valid(&self) -> bool {
        self.key
            .verify(&self.expected_hash[..], &self.sig[..])
            .is_ok()
    }
}
