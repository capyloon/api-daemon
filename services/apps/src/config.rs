use serde::Deserialize;
use std::path::PathBuf;

#[derive(Default, Deserialize, Clone)]
pub struct Config {
    root_path: String,
    data_path: String,
    pub uds_path: String,
    pub cert_type: String,
    pub updater_socket: String,
    pub user_agent: String,
    pub allow_remove_preloaded: bool,
}

impl Config {
    pub fn new(
        root_path: String,
        data_path: String,
        uds_path: String,
        cert_type: String,
        updater_socket: String,
        user_agent: String,
        allow_remove_preloaded: bool,
    ) -> Self {
        let mut config = Self {
            root_path,
            data_path,
            uds_path,
            cert_type,
            updater_socket,
            user_agent,
            allow_remove_preloaded,
        };
        config.resolve_paths();
        config
    }

    pub fn resolve_paths(&mut self) {
        if let Ok(current) = std::env::current_dir() {
            self.root_path = current.join(self.root_path.clone()).display().to_string();
            self.data_path = current.join(self.data_path.clone()).display().to_string();
        }
    }

    pub fn root_path(&self) -> PathBuf {
        PathBuf::from(&self.root_path)
    }

    pub fn data_path(&self) -> PathBuf {
        PathBuf::from(&self.data_path)
    }
}
