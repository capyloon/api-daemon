mod response;

use actix_cors::Cors;
use actix_files::NamedFile;
use actix_web::http::StatusCode;
use actix_web::web::resource;
use actix_web::{get, post, web::Data, App, HttpRequest, HttpResponse, HttpServer};
use clap::{crate_authors, crate_description, crate_version, Clap};
use log::debug;
use parking_lot::RwLock;
use response::{get_download_req_response, get_full_check_response};
use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::Result;
use std::io::{BufReader, Read, Write};
use std::path::Path;
use std::sync::Arc;

const ABOUT_MODE: &str = "server running mode";
const ABOUT_PORT: &str = "server listen port";
const ABOUT_SIZE: &str = "created file size in MB";

#[derive(Clap, PartialEq, Debug, Clone)]
#[clap(
    version = crate_version!(),
    author = crate_authors!(),
    about = crate_description!()
)]
struct Opts {
    #[clap(arg_enum, short, long, default_value = "full", about = ABOUT_MODE)]
    mode: Mode,
    #[clap(short, long, default_value = "10095", about = ABOUT_PORT)]
    port: u16,
    #[clap(short, long, default_value = "3", about = ABOUT_SIZE)]
    size: u8,
}

const GENERATED_PATH: &str = "generated";
const SOURCE_FILE1: &str = "file1";
const SOURCE_FILE2: &str = "file2";

const CUREF: &str = "fota-debug-server-curef";
const V0: &str = "fota-debug-server-v0";
const V1: &str = "fota-debug-server-v1";
const V2: &str = "fota-debug-server-v2";

// const FILE_PATH: &str = "/update.zip";
// const CUREF: &str = "QRD8905-7FOTA";
// const VERSION: &str = "20180925M2";
// const TARGET_VERSION: &str = "20180926N2";

#[derive(Clap, Debug, Copy, Clone, PartialEq)]
pub enum Mode {
    CheckUpdate,
    Error,
    NoPackage,
    Full,
}

impl Default for Mode {
    fn default() -> Mode {
        Mode::Full
    }
}

#[derive(Clone, Debug, Default)]
struct SharedData {
    mode: Mode,
    size: u8,
    port: u16,
    file1: Option<String>,
    file2: Option<String>,
    file1_sha1: Option<String>,
    file2_sha1: Option<String>,
    running_data: Arc<RwLock<RunningData>>,
}

#[derive(Debug, Default)]
struct RunningData {
    check_count: u32,
    query_string: String,
}

fn calculate_sha1<P: AsRef<Path>>(filename: P) -> Result<String> {
    const BUFFER_SIZE: usize = 1024;
    let mut sh = Sha1::new();
    let mut buffer = [0u8; BUFFER_SIZE];
    let file = std::fs::File::open(filename)?;
    let mut reader = BufReader::new(file);
    loop {
        let n = reader.read(&mut buffer)?;
        sh.update(&buffer[..n]);
        if n < BUFFER_SIZE {
            break;
        }
    }
    let result = sh.finalize();
    let mut result_str = String::new();
    for byte in &result {
        result_str.push_str(format!("{:02x}", byte).as_str());
    }
    Ok(result_str)
}

/// Create a file with random data as source file for test, return sha1
fn create_source_file<P: AsRef<Path>>(path: P, size_in_kb: u32) -> Result<String> {
    debug!("Creating source file...");
    let mut file = File::create(&path)?;
    for _ in 0..size_in_kb {
        let bytes: Vec<u8> = (0..1024).map(|_| rand::random::<u8>()).collect();
        file.write_all(&bytes)?;
    }
    debug!("Source file created!");
    calculate_sha1(path)
}

fn get_shared_data(opts: &Opts) -> Result<SharedData> {
    let mut shared_data: SharedData = SharedData::default();
    shared_data.mode = opts.mode;
    shared_data.size = opts.size;
    shared_data.port = opts.port;
    let size = opts.size as u32;

    match opts.mode {
        Mode::Error => return Ok(shared_data),
        Mode::NoPackage => return Ok(shared_data),
        Mode::CheckUpdate => {
            let source_file2_path = format!("{}/{}", GENERATED_PATH, SOURCE_FILE2);
            let source_file2_sha1 = create_source_file(&source_file2_path, size * 1024)?;
            shared_data.file2 = Some(source_file2_path);
            shared_data.file2_sha1 = Some(source_file2_sha1);
        }
        Mode::Full => {}
    }

    let source_file1_path = format!("{}/{}", GENERATED_PATH, SOURCE_FILE1);
    let source_file1_sha1 = create_source_file(&source_file1_path, size * 1024)?;
    shared_data.file1 = Some(source_file1_path);
    shared_data.file1_sha1 = Some(source_file1_sha1);
    Ok(shared_data)
}

#[get("/check.php")]
async fn check_php(data: Data<SharedData>, req: HttpRequest) -> Result<HttpResponse> {
    let check_count = {
        let mut running_data = data.running_data.write();
        running_data.check_count += 1;
        running_data.query_string = req.query_string().to_string();
        running_data.check_count
    };

    let mut target_version = V1;
    let target_file_size = format!("{}", data.size as u32 * 1024 * 1024);
    let mut file_sha1 = &data.file1_sha1;
    match data.mode {
        Mode::CheckUpdate => {
            if check_count % 2 == 0 {
                target_version = V2;
                file_sha1 = &data.file2_sha1;
            }
        }
        Mode::Error => return Ok(HttpResponse::InternalServerError().finish()),
        Mode::NoPackage => return Ok(HttpResponse::NoContent().finish()),
        Mode::Full => {}
    }
    let target_file_sha1: String = match file_sha1 {
        Some(sha1) => sha1.clone(),
        None => "".to_string(),
    };
    let check_response = get_full_check_response(
        CUREF,
        target_version,
        V0,
        &target_file_size,
        &target_file_sha1,
    );
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("text/xml; charset=utf-8")
        .body(check_response))
}

#[post("/download_request.php")]
async fn download_request_php(data: Data<SharedData>) -> Result<HttpResponse> {
    let mut file_path = &data.file1;
    match data.mode {
        Mode::CheckUpdate => {
            let check_count = data.running_data.read().check_count;
            if check_count % 2 == 0 {
                file_path = &data.file2;
            }
        }
        Mode::Error => return Ok(HttpResponse::InternalServerError().finish()),
        Mode::NoPackage => return Ok(HttpResponse::NoContent().finish()),
        Mode::Full => {}
    }
    let target_file_path: String = match file_path {
        Some(file) => format!("/{}", file),
        None => "/not_existed_file".to_string(),
    };
    let slave_addr = format!("127.0.0.1:{}", data.port);
    let download_request_response = get_download_req_response(&target_file_path, &slave_addr);
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("text/xml; charset=utf-8")
        .body(download_request_response))
}

#[get("/api/last_chk_query")]
async fn api_last_chk_query(data: Data<SharedData>) -> Result<String> {
    let running_data = data.running_data.read();
    Ok(running_data.query_string.clone())
}

async fn resource_file1(data: Data<SharedData>) -> Result<NamedFile> {
    match &data.file1 {
        Some(f) => NamedFile::open(&f),
        None => Err(std::io::Error::new(std::io::ErrorKind::NotFound, "file2")),
    }
}

async fn resource_file2(data: Data<SharedData>) -> Result<NamedFile> {
    match &data.file2 {
        Some(f) => NamedFile::open(&f),
        None => Err(std::io::Error::new(std::io::ErrorKind::NotFound, "file2")),
    }
}

#[actix_rt::main]
async fn main() -> Result<()> {
    env_logger::init();

    let opts: Opts = Opts::parse();
    println!(
        "Server mode={:?}, port={}, file size={}MB",
        opts.mode, opts.port, opts.size
    );

    // Re-create generated directory
    let _ = std::fs::remove_dir_all(GENERATED_PATH);
    std::fs::create_dir_all(GENERATED_PATH)?;

    // Create shared file based on different mode
    let shared_data = get_shared_data(&opts)?;
    println!(
        "File sha1, file1={:?}, sha1={:?}, file2={:?}, sha1={:?}",
        &shared_data.file1, &shared_data.file1_sha1, &shared_data.file2, &shared_data.file2_sha1
    );

    // Run server
    let bind_addr = format!("127.0.0.1:{}", opts.port);
    println!("Starting server at http://{}", bind_addr);
    let shared_file1 = shared_data.file1.clone();
    let shared_file2 = shared_data.file2.clone();
    match (shared_file1, shared_file2) {
        (Some(file1), Some(file2)) => {
            HttpServer::new(move || {
                let route_file1: String = format!("/{}", file1);
                let route_file2: String = format!("/{}", file2);
                App::new()
                    .wrap(Cors::new().finish())
                    .service(check_php)
                    .service(download_request_php)
                    .service(api_last_chk_query)
                    .service(resource(&route_file1).to(resource_file1))
                    .service(resource(&route_file2).to(resource_file2))
                    .data(shared_data.clone())
            })
            .bind(&bind_addr)?
            .run()
            .await
        }
        (Some(file1), None) => {
            let route_file1: String = format!("/{}", file1);
            HttpServer::new(move || {
                App::new()
                    .wrap(Cors::new().finish())
                    .service(check_php)
                    .service(download_request_php)
                    .service(api_last_chk_query)
                    .service(resource(&route_file1).to(resource_file1))
                    .data(shared_data.clone())
            })
            .bind(&bind_addr)?
            .run()
            .await
        }
        _ => {
            HttpServer::new(move || {
                App::new()
                    .wrap(Cors::new().finish())
                    .service(check_php)
                    .service(download_request_php)
                    .service(api_last_chk_query)
                    .data(shared_data.clone())
            })
            .bind(&bind_addr)?
            .run()
            .await
        }
    }
}
