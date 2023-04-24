use serde::Deserialize;
#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    storage_path: String,
    metadata_cache_capacity: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            storage_path: "./".into(),
            metadata_cache_capacity: 100,
        }
    }
}

impl Config {
    pub fn new(path: &str, metadata_cache_capacity: usize) -> Self {
        Self {
            storage_path: path.into(),
            metadata_cache_capacity,
        }
    }

    pub fn storage_path(&self) -> String {
        self.storage_path.clone()
    }

    pub fn metadata_cache_capacity(&self) -> usize {
        self.metadata_cache_capacity
    }
}
