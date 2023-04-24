/// Internal representation of a key-based DID.
use crate::generated::common::Did as SidlDid;
use did_key::{
    from_existing_key, generate, Ed25519KeyPair, Fingerprint, KeyMaterial, PatchedKeyPair,
};
use ed25519_zebra::{SigningKey, VerificationKey};
use rusqlite::Row;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use ucan_key_support::ed25519::Ed25519KeyMaterial;

pub struct Did {
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

    pub fn superuser() -> Self {
        Self {
            name: "superuser".into(),
            key_pair: generate::<Ed25519KeyPair>(None),
        }
    }

    pub fn uri(&self) -> String {
        format!("did:key:{}", &self.key_pair.fingerprint())
    }

    pub fn pubkey_b64(&self) -> String {
        base64::encode(self.key_pair.public_key_bytes())
    }

    pub fn privkey_b64(&self) -> String {
        base64::encode(self.key_pair.private_key_bytes())
    }

    pub fn as_ucan_key(&self) -> Ed25519KeyMaterial {
        let pub_key: VerificationKey =
            VerificationKey::try_from(self.key_pair.public_key_bytes().as_slice()).unwrap();
        let mut pk_slice: [u8; 32] = [0; 32];
        let pk_bytes = self.key_pair.private_key_bytes();

        pk_slice[..32].copy_from_slice(&pk_bytes[..32]);

        let private_key: SigningKey = SigningKey::from(pk_slice);
        Ed25519KeyMaterial(pub_key, Some(private_key))
    }

    pub fn removable(&self) -> bool {
        self.name != "superuser"
    }

    pub fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        let pubkey: String = row.get("pubkey")?;
        let privkey: String = row.get("privkey")?;
        let public_key = base64::decode(pubkey).unwrap();
        let private_key = base64::decode(privkey).unwrap();
        let name = row.get("name")?;

        Ok(Self {
            name,
            key_pair: from_existing_key::<Ed25519KeyPair>(&public_key, Some(&private_key)),
        })
    }

    pub fn is_superuser(&self) -> bool {
        self.name == "superuser"
    }
}

impl From<Did> for SidlDid {
    fn from(value: Did) -> SidlDid {
        SidlDid {
            name: value.name.clone(),
            uri: value.uri(),
        }
    }
}

impl From<&Did> for SidlDid {
    fn from(did: &Did) -> Self {
        Self {
            name: did.name.clone(),
            uri: did.uri(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct SerdeDid {
    name: String,
    public_key: String,
    private_key: String,
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
            name: ser.name,
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

#[test]
fn did_superuser() {
    let did = Did::superuser();

    assert_eq!(did.name, "superuser");
    assert!(!did.removable());
}
