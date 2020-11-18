//! Downloader with HAWK authentication supported.

use crate::deviceinfo::DeviceInfo;
use crate::{AccessTokenInfo, Hawk, Method, ServerInfo};
use log::{debug, error};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION, USER_AGENT};
use std::fs;
use std::io;
use std::path::Path;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub enum DownloadError {
    Unauthorized,
    Request(reqwest::Error),
    Http(String),
    Io(io::Error),
}

#[derive(Clone)]
pub struct Downloader {
    client: reqwest::blocking::Client,
    device_info: DeviceInfo,
    server_info: ServerInfo,
    hawk: Hawk,
}

impl Default for Downloader {
    fn default() -> Self {
        let server_info = ServerInfo {
            token_uri: "https://api.kaiostech.com/v3.0/applications/ZW8svGSlaw1ZLCxWZPQA/tokens"
                .into(),
            api_key: "zaP09k7OsOjXEulzSXsd".into(),
            api_uri: String::new(),
        };

        Downloader::create(&server_info)
    }
}

impl Downloader {
    fn create(server_info: &ServerInfo) -> Self {
        Downloader {
            client: reqwest::blocking::Client::new(),
            device_info: DeviceInfo::default(),
            server_info: server_info.clone(),
            hawk: Hawk::default(),
        }
    }

    pub fn set_hawk(&mut self, token_info: AccessTokenInfo) {
        let expires_in = token_info.expires_in;
        self.hawk.valid_until = Instant::now() + Duration::from_secs(expires_in - 60 * 5);
        self.hawk.token_info = Some(token_info);
        self.hawk.is_external = true;
    }

    pub fn has_valid_ext_token(&self) -> bool {
        self.hawk.is_external && self.hawk.has_valid_token()
    }

    fn build_headers(&self, hawk: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Hawk {}", hawk)).unwrap(),
        );
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&format!("KaiOS/{}", self.device_info.os_version)).unwrap(),
        );
        headers.insert(
            HeaderName::from_lowercase(b"kaiapiversion").unwrap(),
            HeaderValue::from_static("3.0"),
        );
        headers.insert(
            HeaderName::from_lowercase(b"kai-device-info").unwrap(),
            HeaderValue::from_str(&format!(
                "imei={},curef={}",
                self.device_info.imei, self.device_info.reference
            ))
            .unwrap(),
        );
        headers
    }

    // Request a resource at a given url and save it to the path.
    pub fn download<P: AsRef<Path>>(&mut self, url: &str, path: P) -> Result<(), DownloadError> {
        debug!("download: {}", url);

        if !self.ensure_token() {
            error!("Unauthorized to download: {}", url);
            return Err(DownloadError::Unauthorized);
        }

        self.server_info.api_uri = url.into();
        match self
            .hawk
            .get_hawk_header(Method::GET, &self.server_info, None)
        {
            Some(hawk) => {
                let mut response = self
                    .client
                    .get(url)
                    .headers(self.build_headers(&hawk))
                    .send()
                    .map_err(DownloadError::Request)?;

                // Check that we didn't receive a HTTP Error
                if !response.status().is_success() {
                    error!("response {}", response.status().as_str());
                    return Err(DownloadError::Http(response.status().as_str().into()));
                }

                let mut file = fs::File::create(path).map_err(DownloadError::Io)?;
                response
                    .copy_to(&mut file)
                    .map_err(DownloadError::Request)?;

                Ok(())
            }
            None => {
                error!("DownloadError: Unauthorized.");
                Err(DownloadError::Unauthorized)
            }
        }
    }

    // Tries to get/update the Hawk token.
    fn ensure_token(&mut self) -> bool {
        // Get a token if needed.
        if !self.hawk.has_valid_token() && self.device_info.is_ready() {
            self.device_info.get_device_info();
            debug!("ensure_token device_info {:?}", &self.device_info);
            self.server_info.try_get_token_uri();
            debug!("ensure_token server_info {:?}", &self.server_info);
            self.hawk
                .get_access_token(&self.device_info, &self.server_info);
        }

        // Download caller is responsible to set token before downloading
        self.hawk.has_valid_token()
    }
}

#[test]
fn download_file_valid_key() {
    use std::env;

    let _ = env_logger::try_init();
    let current = env::current_dir().unwrap();

    let server_info = ServerInfo {
        token_uri: "https://api.kaiostech.com/v3.0/applications/ZW8svGSlaw1ZLCxWZPQA/tokens".into(),
        api_key: "zaP09k7OsOjXEulzSXsd".into(),
        api_uri: String::new(),
    };

    let _test_dir = current.join("test-dir");
    if !_test_dir.exists() {
       fs::create_dir(&_test_dir).unwrap();
    }

    let mut downloader = Downloader::create(&server_info);
    assert!(downloader.ensure_token());

    let _file_path = _test_dir.join("sample.webapp");
    let res = downloader.download(
        "https://api.kaiostech.com/apps/manifest/RZzvAt4g1Je76j4CycaM",
        &_file_path.to_str().unwrap(),
    );
    assert!(res.is_ok());
}
