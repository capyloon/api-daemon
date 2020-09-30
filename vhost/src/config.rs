use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub port: u16,
    pub root_path: String,
    pub cert_path: String,
    pub key_path: String,
    pub csp: String,
}
