//! A simple Downloader
use log::{debug, error};
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug)]
pub enum DownloadError {
    Unauthorized,
    Request(reqwest::Error),
    Http(String),
    Io(io::Error),
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
}

#[test]
fn downloader_download_file() {
    use std::env;

    let _ = env_logger::try_init();
    let current = env::current_dir().unwrap();

    let downloader = Downloader::default();
    let file_path = current.join("test-fixtures/sample.webapp");
    let res = downloader.download(
        "https://seinlin.org/apps/packages/sample/manifest.webapp",
        &file_path.as_path(),
    );
    assert!(res.is_ok());
}
