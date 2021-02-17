/// Configuration file definition.
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Config {
    pub db_path: String,
    pub data_dir: String,
    pub metadata_cache_capacity: usize, // The number of items kept in the LRU cache.
}
