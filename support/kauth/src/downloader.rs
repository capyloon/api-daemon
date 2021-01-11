//! Downloader with HAWK authentication supported.

use crate::deviceinfo::DeviceInfo;
use crate::{AccessTokenInfo, Hawk, Method, ServerInfo};
use log::{debug, error, info};
use nix::unistd;
use reqwest::header::{self, HeaderMap, HeaderName, HeaderValue, AUTHORIZATION, USER_AGENT};
use std::fs;
use std::fs::OpenOptions;
use std::io;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DownloadError {
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Request")]
    Request(reqwest::Error),
    #[error("Http")]
    Http(String),
    #[error("Io")]
    Io(#[from] io::Error),
    #[error("Other")]
    Other(String),
    #[error("Canceled")]
    Canceled,
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
    // Return a tuple with first one to receive result, and second one to
    // cancel download if needed
    // User needs to call receiver.recv_timeout() to get download result
    pub fn download<P: AsRef<Path>>(
        mut self,
        url: &str,
        path: P,
    ) -> (Receiver<Result<(), DownloadError>>, Sender<()>) {
        debug!("download: {}", url);
        let (cancel_sender, canceled_recv) = channel();
        let url_string = String::from(url);
        let file_path_buf = path.as_ref().to_path_buf();
        let (sender, receiver) = channel();

        thread::Builder::new()
            .name("apps_download".into())
            .spawn(move || {
                let result = self.single_download(&url_string, &file_path_buf, canceled_recv);
                debug!("result {:?}", result);
                let _ = sender.send(result);
            })
            .expect("Failed to start downloading thread");

        (receiver, cancel_sender)
    }

    fn single_download<P: AsRef<Path>>(
        &mut self,
        url: &str,
        path: P,
        canceled_recv: Receiver<()>,
    ) -> Result<(), DownloadError> {
        if !self.ensure_token() {
            error!("Unauthorized getting auth token {}", url);
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

                let ct_len = if let Some(val) = response.headers().get(header::CONTENT_LENGTH) {
                    Some(val.to_str().unwrap().parse::<usize>().unwrap())
                } else {
                    None
                };
                debug!("ct_len is  {:?}", ct_len);
                let mut cnt = 0;
                let tmp_file = FileRemover {
                    path: tmp_file_name(),
                };
                debug!("tmp_path is  {:?}", &tmp_file.path);
                let mut file = io::BufWriter::new(get_file_handle(&tmp_file.path, false)?);
                loop {
                    if canceled_recv.try_recv().is_ok() {
                        info!("cancel received while downloading {}", url);
                        return Err(DownloadError::Canceled);
                    }
                    let mut buffer = vec![0; 4 * 1024];
                    let bcount = response.read(&mut buffer[..])?;
                    cnt += bcount;
                    debug!("single_download in loop, cnt is {:?}", cnt);
                    buffer.truncate(bcount);
                    if !buffer.is_empty() {
                        let _ = file.write_all(&buffer)?;
                    } else {
                        break;
                    }
                    if Some(cnt) == ct_len {
                        break;
                    }
                }
                let _ = file.flush().map_err(DownloadError::Io)?;

                fs::copy(&tmp_file.path, path).map_err(DownloadError::Io)?;

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

pub fn get_file_handle(fname: &str, resume_download: bool) -> io::Result<fs::File> {
    if resume_download && Path::new(fname).exists() {
        OpenOptions::new().append(true).open(fname)
    } else {
        OpenOptions::new().write(true).create(true).open(fname)
    }
}

struct FileRemover {
    pub path: String,
}

impl Drop for FileRemover {
    fn drop(&mut self) {
        let _ = fs::remove_file(PathBuf::from(&self.path).as_path());
    }
}

pub fn tmp_file_name() -> String {
    match unistd::mkstemp("/tmp/tempfile_XXXXXX") {
        Ok((_, path)) => path.as_path().to_str().unwrap_or("").into(),
        Err(_) => "".into(),
    }
}

#[test]
fn download_file_valid_key() {
    use std::env;
    use std::sync::mpsc::{channel, Receiver, Sender};

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
    let (result_recv, cancel_sender) = downloader.clone().download(
        "https://api.kaiostech.com/apps/manifest/RZzvAt4g1Je76j4CycaM",
        &_file_path.to_str().unwrap(),
    );

    cancel_sender.send(());

    if let Ok(res) = result_recv.recv_timeout(Duration::from_secs(120)) {
        match res {
            Err(err) => assert_eq!(format!("{:?}", err), format!("{}", "Canceled")),
            _ => assert!(false),
        };
    }

    let (result_recv, cancel_sender) = downloader.download(
        "https://api.kaiostech.com/apps/manifest/RZzvAt4g1Je76j4CycaM",
        &_file_path.to_str().unwrap(),
    );
    if let Ok(res) = result_recv.recv_timeout(Duration::from_secs(120)) {
        assert!(res.is_ok());
    }
}
