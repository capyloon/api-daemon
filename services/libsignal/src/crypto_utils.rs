use crate::generated::common::*;
use common::traits::{SimpleObjectTracker, TrackerId};
use hmac::{Hmac, Mac, NewMac};
use ring::digest;
use sha2::Sha256;

/// HMAC SHA 256 wrapper.
pub struct HmacSha256 {
    ctxt: Hmac<Sha256>,
    id: TrackerId,
}

impl HmacSha256 {
    pub fn new(id: TrackerId, key: &[u8]) -> Option<Self> {
        Hmac::<Sha256>::new_varkey(key)
            .map(|ctxt| Self { ctxt, id })
            .ok()
    }
}

impl HmacSha256Methods for HmacSha256 {
    fn update(&mut self, responder: &HmacSha256UpdateResponder, data: Vec<u8>) {
        self.ctxt.update(&data);
        responder.resolve();
    }

    fn finalize(&mut self, responder: &HmacSha256FinalizeResponder) {
        responder.resolve(self.ctxt.finalize_reset().into_bytes().to_vec());
    }
}

impl SimpleObjectTracker for HmacSha256 {
    fn id(&self) -> TrackerId {
        self.id
    }
}

/// SHA 512 Digest wrapper
pub struct Sha512Digest {
    pub ctxt: digest::Context,
    id: TrackerId,
}

impl Sha512Digest {
    pub fn new(id: TrackerId) -> Self {
        Sha512Digest {
            ctxt: digest::Context::new(&digest::SHA512),
            id,
        }
    }
}

impl SimpleObjectTracker for Sha512Digest {
    fn id(&self) -> TrackerId {
        self.id
    }
}

impl Sha512DigestMethods for Sha512Digest {
    fn update(&mut self, responder: &Sha512DigestUpdateResponder, data: Vec<u8>) {
        self.ctxt.update(&data);
        responder.resolve();
    }

    fn finalize(&mut self, responder: &Sha512DigestFinalizeResponder) {
        responder.resolve(self.ctxt.clone().finish().as_ref().to_vec());
    }
}
