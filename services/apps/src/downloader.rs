//! A simple Downloader
use log::{debug, error};
use nix::unistd;
use reqwest::blocking::Response;
use reqwest::header;
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
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

pub struct Downloader {
    client: reqwest::blocking::Client,
}

impl Default for Downloader {
    fn default() -> Self {
        Downloader {
            client: reqwest::blocking::Client::new(),
        }
    }
}

impl Downloader {
    // Downloads a resource at a given url and save it to the path.
    pub fn download<P: AsRef<Path>>(&self, url: &str, path: P) -> Result<(), DownloadError> {
        debug!("download: {}", url);

        let mut response = self
            .client
            .get(url)
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

    // Downloads a resource at a given url and save it to the path.
    pub fn cancelable_download<P: AsRef<Path>>(
        &self,
        url: &str,
        path: P,
    ) -> (Receiver<Result<(), DownloadError>>, Sender<()>) {
        debug!("download: {}", url);

        let (cancel_sender, canceled_recv) = channel();
        let file_path_buf = path.as_ref().to_path_buf();
        let (sender, receiver) = channel();

        let response = self.client.get(url).send();

        thread::Builder::new()
            .name("apps_download".into())
            .spawn(move || {
                let result = Downloader::single_download(response, &file_path_buf, canceled_recv);
                debug!("result {:?}", result);
                let _ = sender.send(result);
            })
            .expect("Failed to start downloading thread");

        (receiver, cancel_sender)
    }

    fn single_download<P: AsRef<Path>>(
        response: Result<Response, reqwest::Error>,
        path: P,
        canceled_recv: Receiver<()>,
    ) -> Result<(), DownloadError> {
        let mut response = response.map_err(DownloadError::Request)?;

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
                debug!("cancel received while downloading {}", response.url());
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
fn downloader_download_file() {
    use std::env;

    let _ = env_logger::try_init();
    let current = env::current_dir().unwrap();

    let downloader = Downloader::default();
    let file_path = current.join("test-fixtures/sample.webmanifest");
    let res = downloader.download(
        "https://seinlin.org/apps/packages/sample/manifest.webapp",
        &file_path.as_path(),
    );
    assert!(res.is_ok());
}
