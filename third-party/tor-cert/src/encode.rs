//! Code for constructing and signing certificates.
//!
//! Only available when the crate is built with the `encode` feature.

use crate::{
    CertEncodeError, CertExt, Ed25519Cert, Ed25519CertConstructor, ExtType, SignedWithEd25519Ext,
    UnrecognizedExt,
};
use std::time::{Duration, SystemTime};
use tor_bytes::{EncodeResult, Writeable, Writer};
use tor_llcrypto::pk::ed25519;

impl Ed25519Cert {
    /// Return a new `Ed25519CertConstructor` to create and return a new signed
    /// `Ed25519Cert`.
    pub fn constructor() -> Ed25519CertConstructor {
        Default::default()
    }
}

impl Writeable for CertExt {
    /// As Writeable::WriteOnto, but may return an error.
    ///
    /// TODO: Migrate Writeable to provide this interface.
    fn write_onto<B: Writer + ?Sized>(&self, w: &mut B) -> EncodeResult<()> {
        match self {
            CertExt::SignedWithEd25519(pk) => pk.write_onto(w),
            CertExt::Unrecognized(u) => u.write_onto(w),
        }
    }
}

impl Writeable for SignedWithEd25519Ext {
    /// As Writeable::WriteOnto, but may return an error.
    fn write_onto<B: Writer + ?Sized>(&self, w: &mut B) -> EncodeResult<()> {
        // body length
        w.write_u16(32);
        // Signed-with-ed25519-key-extension
        w.write_u8(ExtType::SIGNED_WITH_ED25519_KEY.into());
        // flags = 0.
        w.write_u8(0);
        // body
        w.write_all(self.pk.as_bytes());
        Ok(())
    }
}

impl Writeable for UnrecognizedExt {
    /// As Writeable::WriteOnto, but may return an error.
    fn write_onto<B: Writer + ?Sized>(&self, w: &mut B) -> EncodeResult<()> {
        // We can't use Writer::write_nested_u16len here, since the length field
        // doesn't include the type or the flags.
        w.write_u16(
            self.body
                .len()
                .try_into()
                .map_err(|_| tor_bytes::EncodeError::BadLengthValue)?,
        );
        w.write_u8(self.ext_type.into());
        let flags = if self.affects_validation { 1 } else { 0 };
        w.write_u8(flags);
        w.write_all(&self.body[..]);
        Ok(())
    }
}

impl Ed25519CertConstructor {
    /// Set the approximate expiration time for this certificate.
    ///
    /// (The time will be rounded forward to the nearest hour after the epoch.)
    pub fn expiration(&mut self, expiration: SystemTime) -> &mut Self {
        /// The number of seconds in an hour.
        const SEC_PER_HOUR: u64 = 3600;
        let duration = expiration
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0));
        let exp_hours = duration.as_secs().saturating_add(SEC_PER_HOUR - 1) / SEC_PER_HOUR;
        self.exp_hours = Some(exp_hours.try_into().unwrap_or(u32::MAX));
        self
    }

    /// Set the signing key to be included with this certificate.
    ///
    /// This is optional: you don't need to include the signing key at all.  If
    /// you do, it must match the key that you actually use to sign the
    /// certificate.
    pub fn signing_key(&mut self, key: ed25519::Ed25519Identity) -> &mut Self {
        self.clear_signing_key();
        self.signed_with = Some(Some(key));
        self.extensions
            .get_or_insert_with(Vec::new)
            .push(CertExt::SignedWithEd25519(SignedWithEd25519Ext { pk: key }));

        self
    }

    /// Remove any signing key previously set on this Ed25519CertConstructor.
    pub fn clear_signing_key(&mut self) -> &mut Self {
        self.signed_with = None;
        self.extensions
            .get_or_insert_with(Vec::new)
            .retain(|ext| !matches!(ext, CertExt::SignedWithEd25519(_)));
        self
    }

    /// Encode a certificate into a new vector, signing the result
    /// with `keypair`.
    ///
    /// This function exists in lieu of a `build()` function, since we have a rule that
    /// we don't produce an `Ed25519Cert` except if the certificate is known to be
    /// valid.
    pub fn encode_and_sign(&self, skey: &ed25519::Keypair) -> Result<Vec<u8>, CertEncodeError> {
        use ed25519::Signer;
        let Ed25519CertConstructor {
            exp_hours,
            cert_type,
            cert_key,
            extensions,
            signed_with,
        } = self;

        if let Some(Some(signer)) = &signed_with {
            if *signer != skey.public.into() {
                return Err(CertEncodeError::KeyMismatch);
            }
        }

        let mut w = Vec::new();
        w.write_u8(1); // Version
        w.write_u8(
            cert_type
                .ok_or(CertEncodeError::MissingField("cert_type"))?
                .into(),
        );
        w.write_u32(exp_hours.ok_or(CertEncodeError::MissingField("expiration"))?);
        let cert_key = cert_key
            .clone()
            .ok_or(CertEncodeError::MissingField("cert_key"))?;
        w.write_u8(cert_key.key_type().into());
        w.write_all(cert_key.as_bytes());
        let extensions = extensions.as_ref().map(Vec::as_slice).unwrap_or(&[]);
        w.write_u8(
            extensions
                .len()
                .try_into()
                .map_err(|_| CertEncodeError::TooManyExtensions)?,
        );

        for e in extensions.iter() {
            e.write_onto(&mut w)?;
        }

        let signature = skey.sign(&w[..]);
        w.write(&signature)?;
        Ok(w)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod test {
    use super::*;
    use crate::CertifiedKey;
    use tor_checkable::{SelfSigned, Timebound};
    use tor_llcrypto::util::rand_compat::RngCompatExt;

    #[test]
    fn signed_cert_without_key() {
        let mut rng = rand::thread_rng().rng_compat();
        let keypair = ed25519::Keypair::generate(&mut rng);
        let now = SystemTime::now();
        let day = Duration::from_secs(86400);
        let encoded = Ed25519Cert::constructor()
            .expiration(now + day * 30)
            .cert_key(CertifiedKey::Ed25519(keypair.public.into()))
            .cert_type(7.into())
            .encode_and_sign(&keypair)
            .unwrap();

        let decoded = Ed25519Cert::decode(&encoded).unwrap(); // Well-formed?
        let validated = decoded
            .check_key(Some(&keypair.public.into()))
            .unwrap()
            .check_signature()
            .unwrap(); // Well-signed?
        let cert = validated.check_valid_at(&(now + day * 20)).unwrap();
        assert_eq!(cert.cert_type(), 7.into());
        if let CertifiedKey::Ed25519(found) = cert.subject_key() {
            assert_eq!(found, &keypair.public.into());
        } else {
            panic!("wrong key type");
        }
        assert!(cert.signing_key() == Some(&keypair.public.into()));
    }
}
