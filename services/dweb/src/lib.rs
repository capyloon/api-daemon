pub mod config;
pub mod did;
pub mod generated;
mod handshake;
pub mod http;
pub mod mdns;
pub mod service;
pub mod sidl_ucan;
pub mod storage;

// Helpers that can be reused by multiple services.
use crate::generated::common::Peer;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use common::traits::SharedServiceState;
use core::str::FromStr;
use did_key::{CoreSign, Ed25519KeyPair, Fingerprint};
use ucan::crypto::did::{DidParser, KeyConstructorSlice};
use ucan::crypto::KeyMaterial;
use ucan::ucan::Ucan;

pub const SUPPORTED_UCAN_KEYS: &KeyConstructorSlice = &[
    // https://github.com/multiformats/multicodec/blob/e9ecf587558964715054a0afcc01f7ace220952c/table.csv#L94
    (&[0xed, 0x01], bytes_to_ed25519_key),
];

struct UcanKeyPair {
    key_pair: Ed25519KeyPair,
}

impl UcanKeyPair {
    fn new(bytes: &[u8]) -> Self {
        Self {
            key_pair: Ed25519KeyPair::from_public_key(bytes),
        }
    }
}

#[async_trait]
impl KeyMaterial for UcanKeyPair {
    fn get_jwt_algorithm_name(&self) -> String {
        "EdDSA".into()
    }

    async fn get_did(&self) -> Result<String> {
        Ok(format!("did:key:{}", self.key_pair.fingerprint()))
    }

    async fn sign(&self, payload: &[u8]) -> Result<Vec<u8>> {
        Ok(CoreSign::sign(&self.key_pair, payload))
    }

    async fn verify(&self, payload: &[u8], signature: &[u8]) -> Result<()> {
        CoreSign::verify(&self.key_pair, payload, signature).map_err(|error| anyhow!("{:?}", error))
    }
}

fn bytes_to_ed25519_key(bytes: Vec<u8>) -> Result<Box<dyn KeyMaterial>> {
    Ok(Box::new(UcanKeyPair::new(bytes.as_slice())))
}

pub async fn validate_ucan_token(token: &str) -> Result<Ucan, ()> {
    let ucan = Ucan::from_str(token).map_err(|_| ())?;
    // Parse the token, check time bounds and signature.
    let mut parser = DidParser::new(SUPPORTED_UCAN_KEYS);
    ucan.validate(&mut parser).await.map_err(|_| ())?;
    // Check that the issuer is a known one.
    let dweb_state = crate::service::DWebServiceImpl::shared_state();
    let state = dweb_state.lock();
    let res = if let Ok(Some(did)) = state.dweb_store.did_by_uri(ucan.issuer()) {
        // Check if this ucan is not blocked.
        let not_blocked = did.is_superuser()
            || !state
                .dweb_store
                .is_ucan_blocked(token)
                .map_err(|_| ())?
                .unwrap_or(true);
        if not_blocked {
            Ok(ucan)
        } else {
            Err(())
        }
    } else {
        Err(())
    };
    res
}

// Trait to implement by peer discovery mechanisms.

pub enum DiscoveryError {
    Error,
}

pub trait DiscoveryMechanism {
    fn with_state(state: common::traits::Shared<service::State>) -> Option<Self>
    where
        Self: Sized;

    fn start(&mut self, peer: &Peer) -> Result<(), DiscoveryError>;
    fn stop(&mut self) -> Result<(), DiscoveryError>;
}
