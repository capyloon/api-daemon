use serde::Deserialize;

#[derive(Default, Deserialize, Clone)]
pub struct Config {
    pub root_path: String,
    pub data_path: String,
    pub uds_path: String,
    pub cert_type: String,
    pub updater_socket: String,
    pub user_agent: String,
}
