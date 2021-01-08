use serde::Deserialize;

#[derive(Default, Deserialize, Clone)]
pub struct Config {
    pub socket_path: String,
    pub hints_path: String,
}
