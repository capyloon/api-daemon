/// Internal representation of a key-based DID.
use crate::generated::common::Did as SidlDid;
use did_key::{
    from_existing_key, generate, Ed25519KeyPair, Fingerprint, KeyMaterial, PatchedKeyPair,
};
use serde::{Deserialize, Serialize};

pub(crate) struct Did {
    pub name: String,
    pub key_pair: PatchedKeyPair,
}

impl Clone for Did {
    fn clone(&self) -> Self {
        let public_key = self.key_pair.public_key_bytes();
        let private_key = self.key_pair.private_key_bytes();
        Self {
            name: self.name.clone(),
            key_pair: from_existing_key::<Ed25519KeyPair>(&public_key, Some(&private_key)),
        }
    }
}

impl Did {
    pub fn create(name: &str) -> Self {
        Self {
            name: name.into(),
            key_pair: generate::<Ed25519KeyPair>(None),
        }
    }

    pub fn uri(&self) -> String {
        format!("did:key:{}", &self.key_pair.fingerprint())
    }
}

impl Into<SidlDid> for Did {
    fn into(self) -> SidlDid {
        SidlDid {
            name: self.name.clone(),
            uri: self.uri(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct SerdeDid {
    name: String,
    public_key: String,
    private_key: String,
}

impl From<&Did> for SidlDid {
    fn from(did: &Did) -> Self {
        Self {
            name: did.name.clone(),
            uri: did.uri(),
        }
    }
}

impl From<&Did> for SerdeDid {
    fn from(did: &Did) -> Self {
        Self {
            name: did.name.clone(),
            public_key: base64::encode(did.key_pair.public_key_bytes()),
            private_key: base64::encode(did.key_pair.private_key_bytes()),
        }
    }
}

impl From<SerdeDid> for Did {
    fn from(ser: SerdeDid) -> Self {
        let public_key = base64::decode(ser.public_key).unwrap();
        let private_key = base64::decode(ser.private_key).unwrap();

        Self {
            name: ser.name.clone(),
            key_pair: from_existing_key::<Ed25519KeyPair>(&public_key, Some(&private_key)),
        }
    }
}

#[test]
fn did_roundtrip() {
    let did = Did::create("test");
    let uri1 = did.uri();

    let serialized: SerdeDid = (&did).into();
    let json = serde_json::to_string(&serialized).unwrap();
    let deserialized: SerdeDid = serde_json::from_str(&json).unwrap();

    let new_did = Did::from(deserialized);
    let uri2 = new_did.uri();

    assert_eq!(uri1, uri2);
    assert_eq!(new_did.name, "test");
}
