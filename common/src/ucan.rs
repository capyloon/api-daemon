/// UCAN Helpers
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use did_key::{CoreSign, Ed25519KeyPair, Fingerprint};
use ucan::crypto::did::KeyConstructorSlice;
use ucan::crypto::KeyMaterial;

pub const SUPPORTED_UCAN_KEYS: &KeyConstructorSlice = &[
    // https://github.com/multiformats/multicodec/blob/e9ecf587558964715054a0afcc01f7ace220952c/table.csv#L94
    ([0xed, 0x01], bytes_to_ed25519_key),
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
