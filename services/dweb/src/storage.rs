// Storage of DID identifiers.
// Currently using simple JSON storage, making sure we sync()
// the file after each write.
// The proper solution needs to use some secure storage, which
// will be platform dependent.

use crate::did::{Did, SerdeDid};
use crate::generated::common::Did as SidlDid;
use log::error;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::{Path, PathBuf};

pub(crate) struct DidStorage {
    path: PathBuf,
    dids: Vec<Did>,
}

impl DidStorage {
    pub fn new<T: AsRef<Path>>(path: T) -> Self {
        // Create the repository if it doesn't exist.
        // Will panic if that fails.
        if let Err(err) = create_dir_all(&path) {
            panic!(
                "Failed to create dweb path {} at {}",
                path.as_ref().display(),
                err
            );
        }

        let path = path.as_ref().join("did.json");
        let dids = {
            if let Ok(file) = File::open(&path) {
                let dids: Vec<SerdeDid> = serde_json::from_reader(file).unwrap_or_else(|_| vec![]);
                dids.into_iter().map(Did::from).collect()
            } else {
                vec![]
            }
        };

        let mut res = Self { path, dids };

        // Create a default super user.
        if res.dids.is_empty() {
            res.add(&Did::superuser());
        }

        res
    }

    pub fn add(&mut self, did: &Did) -> bool {
        for item in &self.dids {
            if item.name == did.name {
                return false;
            }
        }

        self.dids.push(did.clone());
        if let Err(err) = self.save() {
            error!("Error saving dids: {}", err);
            return false;
        }
        true
    }

    pub fn remove(&mut self, uri: &str) -> bool {
        let position = &self.dids.iter().position(|item| item.uri() == uri);
        if let Some(index) = position {
            if self.dids[*index].removable {
                self.dids.swap_remove(*index);
                return true;
            }
        }
        false
    }

    pub fn get_all(&self) -> Vec<SidlDid> {
        self.dids.iter().map(|item| item.into()).collect()
    }

    pub fn by_name(&self, name: &str) -> Option<Did> {
        self.dids.iter().find(|item| item.name == name).cloned()
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let items: Vec<SerdeDid> = self.dids.iter().map(|item| item.into()).collect();
        let serialized = serde_json::to_vec(&items).unwrap_or_else(|_| vec![]);

        let mut file = File::create(&self.path)?;
        file.write_all(&serialized)?;
        file.sync_all()?;
        Ok(())
    }
}
