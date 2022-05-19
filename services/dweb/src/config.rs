use serde::Deserialize;
#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    storage_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            storage_path: "./".into(),
        }
    }
}

impl Config {
    pub fn new(path: &str) -> Self {
        Self {
            storage_path: path.into(),
        }
    }

    pub fn storage_path(&self) -> String {
        self.storage_path.clone()
    }
}
