//! Downloader with HAWK authentication supported.

use hyper::{body::HttpBody, Body, Client, Request};
use hyperlocal::{UnixClientExt, Uri};
use log::{debug, error, info};
use reqwest::header::{self, HeaderMap};
use std::env::temp_dir;
use std::fs;
use std::io;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::runtime::Runtime;
use url::Url;

#[derive(Debug)]
pub enum DownloaderInfo {
    Etag(String),
    Progress(u8),
    Done,
}

#[derive(Error, Debug)]
pub enum DownloadError {
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Reqwest")]
    Reqwest(reqwest::Error),
    #[error("Hyper")]
    Hyper(hyper::Error),
    #[error("Http")]
    Http(String),
    #[error("Io")]
    Io(#[from] io::Error),
    #[error("Other")]
    Other(String),
    #[error("Canceled")]
    Canceled,
}

impl PartialEq for DownloadError {
    fn eq(&self, right: &DownloadError) -> bool {
        format!("{:?}", self) == format!("{:?}", right)
    }
}

struct DownloadProgress {
    progress_sender: Sender<Result<DownloaderInfo, DownloadError>>,
    content_length: Option<usize>,
    downloaded: usize,
    progress: u8, // In %
    last_check: SystemTime,
    interval: Duration,
}

impl DownloadProgress {
    fn new(
        progress_sender: Sender<Result<DownloaderInfo, DownloadError>>,
        content_length: Option<usize>,
    ) -> Self {
        Self {
            progress_sender,
            content_length,
            downloaded: 0,
            progress: 0,
            last_check: UNIX_EPOCH,
            interval: Duration::from_secs(1),
        }
    }

    fn update(&mut self, amount: usize) {
        self.downloaded += amount;

        let now = SystemTime::now();
        if let Some(full_size) = self.content_length {
            if full_size > 0 && now - self.interval >= self.last_check {
                self.last_check = now;
                let current_progress =
                    f64::trunc(((self.downloaded as f64) / (full_size as f64)) * 100.0) as u8;

                if current_progress > self.progress {
                    self.progress = current_progress;
                    debug!("Progress send {:?}", self.progress);
                    let _ = self
                        .progress_sender
                        .send(Ok(DownloaderInfo::Progress(self.progress)));
                }
            }
        }
    }

    fn downloaded(&self) -> usize {
        self.downloaded
    }

    fn done(&self) {
        // If the downloading finishs within 1 second,
        // We could miss the last 100 progress, send extra here if not sent
        if self.progress != 100 {
            let _ = self.progress_sender.send(Ok(DownloaderInfo::Progress(100)));
        }
    }
}

// TODO: move to the config.
#[cfg(not(target_os = "android"))]
static IPFS_SOCKET_PATH: &str = "/tmp/ipfsd.http";
#[cfg(target_os = "android")]
static IPFS_SOCKET_PATH: &str = "/dev/socket/ipfsd.http";

#[derive(Clone)]
pub struct Downloader {
    client: reqwest::blocking::Client,
}

impl Downloader {
    pub fn new(user_agent: &str, lang: &str) -> Result<Self, DownloadError> {
        let mut headers = header::HeaderMap::new();
        match header::HeaderValue::from_str(lang) {
            Ok(header) => headers.insert(header::ACCEPT_LANGUAGE, header),
            _ => headers.insert(
                header::ACCEPT_LANGUAGE,
                header::HeaderValue::from_static("en-US"),
            ),
        };

        let client = reqwest::blocking::Client::builder()
            .user_agent(user_agent)
            .default_headers(headers)
            .gzip(true)
            .build()
            .map_err(DownloadError::Reqwest)?;

        Ok(Downloader { client })
    }

    // Reqwest a resource at a given url and save it to the path.
    // Return a tuple with first one to receive result, and second one to
    // cancel download if needed
    // User needs to call receiver.recv_timeout() to get download result
    // The etag, if exists, is returned when success.
    pub fn download<P: AsRef<Path>>(
        mut self,
        url: &Url,
        path: P,
        extra_headers: Option<HeaderMap>,
    ) -> (Receiver<Result<DownloaderInfo, DownloadError>>, Sender<()>) {
        debug!("download: {}", url.as_str());
        let url = url.clone();
        let (cancel_sender, canceled_recv) = channel();
        let file_path_buf = path.as_ref().to_path_buf();
        let (sender, receiver) = channel();

        thread::Builder::new()
            .name("apps_download".into())
            .spawn(move || {
                let scheme = url.scheme();
                let result = match scheme {
                    "http" | "https" => self.single_download(
                        &url,
                        &file_path_buf,
                        canceled_recv,
                        sender.clone(),
                        extra_headers,
                    ),
                    "ipfs" | "ipns" | "tile" => self.ipfs_download(
                        &url,
                        &file_path_buf,
                        canceled_recv,
                        sender.clone(),
                        extra_headers,
                        IPFS_SOCKET_PATH,
                    ),
                    _ => Err(DownloadError::Other("UnsupportedUrl".into())),
                };
                debug!("result {:?}", result);
                let _ = sender.send(result);
            })
            .expect("Failed to start downloading thread");

        (receiver, cancel_sender)
    }

    fn ipfs_download<P: AsRef<Path>>(
        &mut self,
        url: &Url,
        path: P,
        canceled_recv: Receiver<()>,
        progress_sender: Sender<Result<DownloaderInfo, DownloadError>>,
        extra_headers: Option<HeaderMap>,
        socket_path: &str,
    ) -> Result<DownloaderInfo, DownloadError> {
        let rt = Runtime::new().unwrap();

        rt.block_on(async move {
            // Build the UDS path from the URL.
            let parts: Vec<&str> = url.as_str().split("://").collect();
            let scheme = match url.scheme() {
                "ipfs" | "tile" => "ipfs",
                "ipns" => "ipns",
                _ => return Err(DownloadError::Other("UnsupportedUrl".into())),
            };

            let ipfs_url = format!("/{}/{}", scheme, parts[1]);
            debug!("ipfs_url is {}", ipfs_url);
            let uds_url = Uri::new(socket_path, &ipfs_url);

            let client = Client::unix();

            let mut builder = Request::builder().method("GET").uri(uds_url);

            if let Some(headers) = extra_headers {
                for (maybe_name, value) in headers {
                    if let Some(name) = maybe_name {
                        builder = builder.header(name, value);
                    }
                }
            }
            let req = builder
                .body(Body::empty())
                .map_err(|e| DownloadError::Other(e.to_string()))?;

            let mut response = client.request(req).await.map_err(DownloadError::Hyper)?;

            let content_length = if let Some(val) = response.headers().get(header::CONTENT_LENGTH) {
                match val.to_str() {
                    Ok(len) => Some(len.parse::<usize>().unwrap_or(0)),
                    _ => Some(0),
                }
            } else {
                None
            };

            let mut progress = DownloadProgress::new(progress_sender.clone(), content_length);

            let mut tmp_file = get_tmp_file()?;
            let tmp_file_path = tmp_file.path().to_string();
            debug!("tmp_path is  {:?}", &tmp_file_path);
            let mut file = io::BufWriter::new(tmp_file.inner());

            while let Some(next) = response.data().await {
                if canceled_recv.try_recv().is_ok() {
                    info!("cancel received while downloading {}", url.as_str());
                    return Err(DownloadError::Canceled);
                }

                let chunk = next.map_err(DownloadError::Hyper)?;
                progress.update(chunk.len());
                file.write_all(&chunk)?;
            }

            progress.done();

            if let Some(val) = response.headers().get(header::ETAG) {
                if let Ok(etag) = val.to_str() {
                    let _ = progress_sender.send(Ok(DownloaderInfo::Etag(etag.into())));
                }
            }

            file.flush().map_err(DownloadError::Io)?;

            fs::copy(&tmp_file_path, path).map_err(DownloadError::Io)?;

            Ok(DownloaderInfo::Done)
        })
    }

    fn single_download<P: AsRef<Path>>(
        &mut self,
        url: &Url,
        path: P,
        canceled_recv: Receiver<()>,
        progress_sender: Sender<Result<DownloaderInfo, DownloadError>>,
        extra_headers: Option<HeaderMap>,
    ) -> Result<DownloaderInfo, DownloadError> {
        let mut response = self
            .client
            .get(url.clone())
            .headers(extra_headers.unwrap_or_default())
            .send()
            .map_err(DownloadError::Reqwest)?;

        // Check that we didn't receive a HTTP Error
        if !response.status().is_success() {
            error!("response {}", response.status().as_str());
            return Err(DownloadError::Http(response.status().as_str().into()));
        }

        let content_length = if let Some(val) = response.headers().get(header::CONTENT_LENGTH) {
            match val.to_str() {
                Ok(len) => Some(len.parse::<usize>().unwrap_or(0)),
                _ => Some(0),
            }
        } else {
            None
        };

        let mut progress = DownloadProgress::new(progress_sender.clone(), content_length);

        let mut tmp_file = get_tmp_file()?;
        let tmp_file_path = tmp_file.path().to_string();
        debug!("tmp_path is  {:?}", &tmp_file_path);
        let mut file = io::BufWriter::new(tmp_file.inner());
        loop {
            if canceled_recv.try_recv().is_ok() {
                info!("cancel received while downloading {}", url.as_str());
                return Err(DownloadError::Canceled);
            }

            let mut buffer = vec![0; 4 * 1024];
            let bcount = response.read(&mut buffer[..])?;

            progress.update(bcount);

            buffer.truncate(bcount);
            if !buffer.is_empty() {
                file.write_all(&buffer)?;
            } else {
                break;
            }
            if let Some(max) = content_length {
                if max == progress.downloaded() {
                    break;
                }
            }
        }

        progress.done();

        file.flush().map_err(DownloadError::Io)?;

        fs::copy(&tmp_file_path, path).map_err(DownloadError::Io)?;

        if let Some(val) = response.headers().get(header::ETAG) {
            if let Ok(etag) = val.to_str() {
                let _ = progress_sender.send(Ok(DownloaderInfo::Etag(etag.into())));
            }
        }

        Ok(DownloaderInfo::Done)
    }
}

pub fn get_tmp_file() -> io::Result<mkstemp::TempFile> {
    let mut dir = temp_dir();
    dir.push("tempfile_XXXXXX");
    let file_path = dir
        .into_os_string()
        .into_string()
        .map_err(|_| io::ErrorKind::NotFound)?;
    mkstemp::TempFile::new(&file_path, true)
}

#[cfg(test)]
mod test {
    use actix_web::http::header::{self, HeaderValue};
    /// This simple server is for apps-service client test.
    /// The server hosts applications including manifest.webmanifest and zip package
    /// Under /apps. Hawk authentication is required and only GET method is supported.
    /// For test purpose, client uses fixed mock id and key to generate Hawk header.
    /// kid: "FGFYvY+/4XwTYIX9nVi+sXj5tPA=", mac_key: "p7cI80SwX+gmX0G+T938agWAV1eR9wrpCR9JgsoIIlk="
    use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer};
    use hawk::mac::Mac;
    use hawk::{Header, Key, RequestBuilder, SHA256};
    use mime_guess::{Mime, MimeGuess};
    use std::env;
    use vhost_server::etag::*;

    use crate::downloader::*;
    use kauth::{AccessTokenInfo, Hawk, Method};
    use log::{debug, error};
    use reqwest::header::{HeaderMap, HeaderName, AUTHORIZATION};
    use std::collections::HashMap;
    use std::fs::{self, File};
    use std::io::Read;
    use std::path::Path;
    use std::thread;
    use std::time::{Duration, Instant};

    fn mime_type_for(file_name: &str) -> Mime {
        MimeGuess::from_path(file_name).first_or_octet_stream()
    }

    fn maybe_not_modified(
        if_none_match: Option<&HeaderValue>,
        etag: &str,
        mime: &Mime,
    ) -> Option<HttpResponse> {
        // Check if we have an etag from the If-None-Match header.
        if let Some(if_none_match) = if_none_match {
            if let Ok(value) = if_none_match.to_str() {
                if etag == value {
                    let mut resp304 = HttpResponse::NotModified();
                    let builder = resp304
                        .content_type(mime.as_ref())
                        .insert_header(("ETag", etag.to_string()));

                    return Some(builder.finish());
                }
            }
        }
        None
    }

    fn response_from_file(req: HttpRequest, app: &str, name: &str) -> HttpResponse {
        let name_string = format!("test-fixtures/test-server-apps/{}/{}", app, name);
        let path = Path::new(&name_string);
        if let Ok(mut file) = File::open(path) {
            // Check if we can return NotModified without reading the file content.
            let if_none_match = req.headers().get(header::IF_NONE_MATCH);
            let etag = Etag::for_file(&file);
            let file_name = path
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new(""))
                .to_string_lossy();
            let mime = mime_type_for(&file_name);
            if let Some(response) = maybe_not_modified(if_none_match, &etag, &mime) {
                return response;
            }

            let mut buf = vec![];
            if let Err(err) = file.read_to_end(&mut buf) {
                error!("Failed to read {} : {}", path.to_string_lossy(), err);
                return HttpResponse::InternalServerError().finish();
            }

            HttpResponse::Ok()
                .insert_header(("ETag", etag.to_string()))
                .content_type(mime.as_ref())
                .body(buf)
        } else {
            HttpResponse::NotFound().finish()
        }
    }

    static MAC_KEY: &str = "p7cI80SwX+gmX0G+T938agWAV1eR9wrpCR9JgsoIIlk=";
    // This UA is defined in daemon/config.toml.
    static EXPECTED_UA: &str = "Mozilla/5.0 (Mobile; rv:84.0) Gecko/84.0 Firefox/84.0 KAIOS/3.0";

    fn check_ua(req: &HttpRequest) -> bool {
        match req.headers().get(::actix_web::http::header::USER_AGENT) {
            Some(value) => match value.to_str() {
                Ok(ua) => ua == EXPECTED_UA,
                Err(_) => false,
            },
            None => false,
        }
    }

    fn validate(req: &HttpRequest) -> bool {
        match req.headers().get(::actix_web::http::header::AUTHORIZATION) {
            Some(header_value) => match header_value.to_str() {
                Ok(value) => {
                    let values: Vec<_> = value.split(',').map(|e| e.trim()).collect();
                    debug!("AUTHORIZATION is {:?}", values.clone());
                    let mut hawk_auth: HashMap<String, String> = HashMap::new();
                    // token_type: "hawk", scope: "u|core:cruds sc#apps:rs sc#metrics:c payment#products:rs payment#purchases:crud simcustm#pack:s simcustm#packfile:r payment#transactions:cr payment#prices:s payment#options:s", expires_in: 604800, kid: "FGFYvY+/4XwTYIX9nVi+sXj5tPA=", mac_key: "p7cI80SwX+gmX0G+T938agWAV1eR9wrpCR9JgsoIIlk=", mac_algorithm: "hmac-sha-256" }
                    //["Hawk id=\"FGFYvY+/4XwTYIX9nVi+sXj5tPA=\"", "ts=\"1611717940\"", "nonce=\"SrnmiS6u9dckTg==\"", "mac=\"gVH14LHIxSTD/Oq7+MsFCpxHzafWRDSEvXlGFnpQAzM=\"", "hash=\"\""]
                    for item in values.iter() {
                        if let Some(index) = item.find('=') {
                            let key = item[0..index].replace(" ", "");
                            let value = item[index + 1..item.len()].replace("\"", "");
                            hawk_auth.insert(key, value);
                        }
                    }
                    debug!("hawk_auth is {:?}", hawk_auth);
                    let id = hawk_auth.get("Hawkid").unwrap();
                    let mac_string = hawk_auth.get("mac").unwrap();
                    let mac = Mac::from(base64::decode(&mac_string).unwrap());
                    let nounce = hawk_auth.get("nonce").unwrap();
                    let hdr = Header::new(
                        Some(id.as_str()),
                        Some(
                            UNIX_EPOCH
                                + Duration::new(
                                    hawk_auth.get("ts").unwrap().parse::<u64>().unwrap(),
                                    0,
                                ),
                        ),
                        Some(nounce.as_str()),
                        Some(mac),
                        None,
                        None,
                        None,
                        None,
                    )
                    .unwrap();

                    let port = req
                        .headers()
                        .get("X-Port")
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .parse()
                        .unwrap();
                    let request =
                        RequestBuilder::new("GET", "localhost", port, req.path()).request();

                    let key = Key::new(base64::decode(MAC_KEY).unwrap(), SHA256).unwrap();
                    let one_week_in_secs = 7 * 24 * 60 * 60;

                    request.validate_header(&hdr, &key, Duration::from_secs(one_week_in_secs))
                }
                Err(_) => false,
            },
            None => false,
        }
    }

    async fn apps_responses(req: HttpRequest, params: web::Path<(String, String)>) -> HttpResponse {
        let (app, name) = params.as_ref();
        // For cancel API test
        std::thread::sleep(std::time::Duration::from_millis(200));
        if !check_ua(&req) {
            return HttpResponse::BadRequest().finish();
        }
        // Do not check the authorization header for pwa.
        if app != "pwa" && !validate(&req) {
            return HttpResponse::Unauthorized().finish();
        }
        response_from_file(req, &app, &name)
    }

    fn launch_server(port: u16) {
        env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");

        let server = HttpServer::new(|| {
            App::new()
                .wrap(middleware::Logger::default())
                .service(
                    web::resource("/test-fixtures/test-server-apps/{app}/{name:[^{}]+}")
                        .route(web::get().to(apps_responses)),
                )
                .service(web::scope("/").route("*", web::post().to(HttpResponse::MethodNotAllowed)))
        })
        .disable_signals()
        .bind(format!("localhost:{}", port))
        .unwrap()
        .run();

        let _ = actix_rt::Runtime::new().unwrap().block_on(async {
            let _ = server
                .await
                .map_err(|e| error!("apps test server exit with error: {:?}", e));
        });
    }

    fn start_server(port: u16) {
        thread::Builder::new()
            .name(format!("download test server on port {}", port))
            .spawn(move || {
                launch_server(port);
            })
            .expect("Failed to start server");

        thread::sleep(std::time::Duration::from_secs(3));
    }

    #[test]
    fn download_304() {
        use std::env;

        let _ = env_logger::try_init();
        let current = env::current_dir().unwrap();

        start_server(3429);

        let user_agent = "Mozilla/5.0 (Mobile; rv:84.0) Gecko/84.0 Firefox/84.0 KAIOS/3.0";
        let lang = "en-US";
        let _test_dir = current.join("test-fixtures").join("downloader-test");
        if !_test_dir.exists() {
            fs::create_dir(&_test_dir).unwrap();
        }
        let url = Url::parse(
            "http://localhost:3429/test-fixtures/test-server-apps/ciautotest/manifest.webmanifest",
        )
        .ok();
        let mut hawk = Hawk::default();
        let token_info = AccessTokenInfo {
            kid: "FGFYvY+/4XwTYIX9nVi+sXj5tPA=".into(),
            mac_key: "p7cI80SwX+gmX0G+T938agWAV1eR9wrpCR9JgsoIIlk=".into(),
            expires_in: 600,
        };
        let expires_in = token_info.expires_in;
        hawk.valid_until = Instant::now() + Duration::from_secs(expires_in);
        hawk.token_info = Some(token_info);

        let hawk_str = hawk
            .get_hawk_header(Method::GET, url.clone(), None)
            .unwrap();
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Hawk {}", hawk_str)).unwrap(),
        );
        headers.insert("X-Port", HeaderValue::from_str("3429").unwrap());

        let downloader = Downloader::new(user_agent, lang).unwrap();

        let _file_path = _test_dir.join("sample.webapp");
        let (result_recv, _) = downloader.clone().download(
            &url.clone().unwrap(),
            &_file_path.to_str().unwrap(),
            Some(headers.clone()),
        );

        let mut etag = String::new();
        let mut progress = 0;
        loop {
            if let Ok(res) = result_recv.recv_timeout(Duration::from_secs(120)) {
                match res {
                    Err(_) => assert!(false),
                    Ok(result) => match result {
                        DownloaderInfo::Progress(prog) => {
                            progress = prog;
                        }
                        DownloaderInfo::Etag(tag) => {
                            etag = tag;
                        }
                        DownloaderInfo::Done => {
                            break;
                        }
                    },
                };
            }
        }

        assert_eq!(etag.is_empty(), false);
        assert_eq!(progress, 100);

        headers.insert(
            HeaderName::from_lowercase(b"if-none-match").unwrap(),
            HeaderValue::from_str(&etag).unwrap(),
        );

        let (result_recv, _) = downloader.clone().download(
            &url.clone().unwrap(),
            &_file_path.to_str().unwrap(),
            Some(headers),
        );

        if let Ok(result) = result_recv.recv_timeout(Duration::from_secs(120)) {
            if let Err(err) = result {
                assert!(err == DownloadError::Http("304".into()));
            } else {
                assert!(false);
            }
        } else {
            assert!(false);
        }
    }

    #[test]
    fn cancel_download_file_valid_key() {
        use std::env;
        let _ = env_logger::try_init();
        let current = env::current_dir().unwrap();

        start_server(3430);

        let user_agent = "Mozilla/5.0 (Mobile; rv:84.0) Gecko/84.0 Firefox/84.0 KAIOS/3.0";
        let lang = "en-US";

        let _test_dir = current.join("test-fixtures").join("downloader-test");
        if !_test_dir.exists() {
            fs::create_dir(&_test_dir).unwrap();
        }

        let url = Url::parse(
            "http://localhost:3430/test-fixtures/test-server-apps/ciautotest/manifest.webmanifest",
        )
        .ok();
        let mut hawk = Hawk::default();
        let token_info = AccessTokenInfo {
            kid: "FGFYvY+/4XwTYIX9nVi+sXj5tPA=".into(),
            mac_key: "p7cI80SwX+gmX0G+T938agWAV1eR9wrpCR9JgsoIIlk=".into(),
            expires_in: 600,
        };
        let expires_in = token_info.expires_in;
        hawk.valid_until = Instant::now() + Duration::from_secs(expires_in);
        hawk.token_info = Some(token_info);

        let hawk_str = hawk
            .get_hawk_header(Method::GET, url.clone(), None)
            .unwrap();
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Hawk {}", hawk_str)).unwrap(),
        );
        headers.insert("X-Port", HeaderValue::from_str("3430").unwrap());

        let downloader = Downloader::new(user_agent, lang).unwrap();

        assert!(hawk.has_valid_token());

        let _file_path = _test_dir.join("sample.webapp");
        let (result_recv, cancel_sender) = downloader.clone().download(
            &url.clone().unwrap(),
            &_file_path.to_str().unwrap(),
            Some(headers.clone()),
        );

        let _ = cancel_sender.send(());

        if let Ok(res) = result_recv.recv_timeout(Duration::from_secs(120)) {
            match res {
                Err(err) => assert_eq!(format!("{:?}", err), format!("{}", "Canceled")),
                _ => assert!(false),
            };
        }

        let (result_recv, _cancel_sender) = downloader.download(
            &url.clone().unwrap(),
            &_file_path.to_str().unwrap(),
            Some(headers),
        );
        if let Ok(res) = result_recv.recv_timeout(Duration::from_secs(120)) {
            assert!(res.is_ok());
        }
    }
}
