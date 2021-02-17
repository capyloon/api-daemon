// Configuration parameters.
// This is using a toml file with the following syntax:
// [general]
// host = "0.0.0.0"
// port = 8081
//
// [http]
// root_path = /path/to/files

#[cfg(feature = "apps-service")]
use apps_service::config::Config as AppsConfig;
use common::traits::EmptyConfig;
#[cfg(feature = "contentmanager-service")]
use contentmanager_service::config::Config as CmConfig;
use serde::{Deserialize, Deserializer};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Custom Error")]
    Custom,
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
        Ok(Error::Custom)
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
    pub apps_service: AppsConfig,
    #[cfg(feature = "procmanager-service")]
    pub procmanager_service: procmanager_service::config::Config,
    #[cfg(feature = "contentmanager-service")]
    pub content_manager: CmConfig,
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
            apps_service: AppsConfig::new(
                "".into(),
                "".into(),
                "".into(),
                "".into(),
                "".into(),
                false,
            ),
            #[cfg(feature = "procmanager-service")]
            procmanager_service: procmanager_service::config::Config {
                socket_path: "".into(),
                hints_path: "".into(),
            },
        }
    }
}

impl From<&Config> for EmptyConfig {
    fn from(_c: &Config) -> EmptyConfig {
        EmptyConfig
    }
}

#[cfg(feature = "apps-service")]
impl From <&Config> for AppsConfig {
    fn from(c: &Config) -> AppsConfig {
        c.apps_service.clone()
    }
}

#[cfg(feature = "contentmanager-service")]
impl Into<CmConfig> for &Config {
    fn into(self) -> CmConfig {
        self.content_manager.clone()
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
        assert!(!config.general.verbose_log);
        assert_eq!(config.general.remote_services_config, "remote_config.toml");
        assert_eq!(config.general.remote_services_path, "./remote/");
        assert_eq!(config.http.root_path, "/tmp");
        #[cfg(feature = "apps-service")]
        {
            use std::path::PathBuf;
            assert_eq!(
                config.apps_service.root_path(),
                PathBuf::from("/tmp/test-fixtures/webapps")
            );
            assert_eq!(config.apps_service.data_path(), PathBuf::from("/tmp/apps"));
            assert_eq!(config.apps_service.uds_path, "/tmp/uds_tmp.sock");
            assert_eq!(config.apps_service.cert_type, "test");
        }
        #[cfg(feature = "procmanager-service")]
        {
            assert_eq!(
                config.procmanager_service.socket_path,
                "/tmp/b2gkiller_hints"
            );
            assert_eq!(config.procmanager_service.hints_path, "/tmp/prochints.dat");
        }
    }
}
