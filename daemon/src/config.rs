// Configuration parameters.
// This is using a toml file with the following syntax:
// [general]
// host = "0.0.0.0"
// port = 8081
//
// [http]
// root_path = /path/to/files

use serde::{Deserialize, Deserializer};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Custom Error")]
    CustomError,
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("IO Error")]
    Io(#[from] ::std::io::Error),
}

type Result<T> = ::std::result::Result<T, Error>;

impl<'de> Deserialize<'de> for Error {
    fn deserialize<D>(_deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Error::CustomError)
    }
}

#[derive(Clone, Deserialize)]
pub struct GeneralConfig {
    pub host: String,
    pub port: u16,
    pub message_max_time: u32,
    pub verbose_log: bool,
    pub log_path: String,
    pub remote_services_config: String,
    pub remote_services_path: String,
    pub socket_path: Option<String>,
}

#[derive(Clone, Deserialize)]
pub struct HttpConfig {
    pub root_path: String,
}

#[derive(Clone, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub http: HttpConfig,
    #[cfg(feature = "virtual-host")]
    pub vhost: vhost_server::config::Config,
    #[cfg(feature = "apps-service")]
    pub apps_service: apps_service::config::Config,
}

impl Config {
    pub fn from_file<P: Into<PathBuf>>(path: P) -> Result<Self> {
        let mut file = File::open(path.into().as_ref() as &Path)?;
        let mut source = String::new();
        file.read_to_string(&mut source)?;
        toml::from_str(&source).map_err(|e| e.into())
    }

    #[cfg(test)]
    pub fn test_on_port(port: u16) -> Self {
        Config {
            general: GeneralConfig {
                host: "0.0.0.0".into(),
                port,
                message_max_time: 10,
                verbose_log: false,
                log_path: "/tmp".into(),
                remote_services_config: "./remote_services.toml".into(),
                remote_services_path: "./remote".into(),
                socket_path: Some("/tmp/api-daemon-uds".into()),
            },
            http: HttpConfig {
                root_path: "./tests/data".into(),
            },
            #[cfg(feature = "virtual-host")]
            vhost: vhost_server::config::Config {
                root_path: "".into(),
                csp: "".into(),
            },
            #[cfg(feature = "apps-service")]
            apps_service: apps_service::config::Config {
                root_path: "".into(),
                data_path: "".into(),
                uds_path: "".into(),
                cert_type: "".into(),
                user_agent: "".into(),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::Config;

    #[test]
    fn unknown_config() {
        let config = Config::from_file("unknown_test_config.toml");
        assert!(config.is_err());
    }

    #[test]
    fn invalid_config() {
        let config = Config::from_file("invalid_test_config.toml");
        assert!(config.is_err());
    }

    #[test]
    fn valid_config() {
        let config = Config::from_file("valid_test_config.toml").unwrap();
        assert_eq!(config.general.host, "0.0.0.0");
        assert_eq!(config.general.port, 8081);
        assert_eq!(config.general.message_max_time, 10);
        assert_eq!(config.general.verbose_log, false);
        assert_eq!(config.general.remote_services_config, "remote_config.toml");
        assert_eq!(config.general.remote_services_path, "./remote/");
        assert_eq!(config.http.root_path, "/tmp");
        #[cfg(feature = "apps-service")]
        {
            assert_eq!(config.apps_service.root_path, "/tmp/test-fixtures/webapps");
            assert_eq!(config.apps_service.data_path, "/tmp/apps");
            assert_eq!(config.apps_service.uds_path, "/tmp/uds_tmp.sock");
            assert_eq!(config.apps_service.cert_type, "test");
        }
    }
}
