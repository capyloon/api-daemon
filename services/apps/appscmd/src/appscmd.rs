//! A command line program to interact with app service daemon.

#[macro_use]
extern crate prettytable;

mod adb;
mod zip_utils;

use adb::{AdbDevice, AdbError};
use clap::{crate_version, App, Arg, SubCommand};
use dirs::home_dir;
use log::error;
use prettytable::{format, Table};
use serde::{Deserialize, Serialize};
use std::env;
use std::io::{BufReader, Read, Write};
use std::net::Shutdown;
use std::net::TcpStream;
#[cfg(not(target_os = "windows"))]
use std::os::unix::net::UnixStream;
use std::path::Path;
use thiserror::Error;
use tinyfiledialogs::select_folder_dialog;
use zip_utils::create_zip_for_dir;

static FORWARDING_PORT: u16 = 6001;

#[derive(Deserialize, Debug)]
struct Response {
    name: String,
    success: Option<String>,
    error: Option<String>,
}

#[derive(Serialize, Debug)]
struct Request {
    cmd: String,
    param: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct AppsObject {
    pub name: String,
    pub install_state: String,
    pub manifest_url: String,
    pub status: String,
    pub update_state: String,
    pub update_url: String,
}

struct CmdOptions {
    is_json_output: bool,
    socket_path: Option<String>,
}

#[derive(Error, Debug)]
pub(crate) enum CmdLineError {
    #[error("Zip Error {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Json Error {0}")]
    Json(#[from] serde_json::Error),
    #[error("I/O Error {0}")]
    Io(#[from] std::io::Error),
    #[error("ADB Error {0}")]
    Adb(#[from] AdbError),
    #[error("Command `{0}` failed: {1}")]
    FailedCommand(String, String),
    #[error("Zipper Error {0}")]
    Zipper(#[from] crate::zip_utils::ZipperError),
    #[error("Error: {0}")]
    Other(String),
}

fn send_request<S: Read + Write + Copy>(
    opts: &CmdOptions,
    request: &Request,
    mut stream: S,
) -> Result<(), CmdLineError> {
    serde_json::to_writer(stream, request)?;

    stream.write_all(b"\r\n")?;
    stream.flush()?;

    let stream_reader = BufReader::new(stream);

    // Read the response.
    let mut deserializer = serde_json::Deserializer::from_reader(stream_reader);
    let response = Response::deserialize(&mut deserializer)?;
    if response.success.is_some() {
        handle_success(opts, response)
    } else if let Some(error) = response.error {
        Err(CmdLineError::FailedCommand(response.name, error))
    } else {
        Err(CmdLineError::Other("MissingSuccessErrorField".into()))
    }
}

fn handle_success(opts: &CmdOptions, response: Response) -> Result<(), CmdLineError> {
    if response.name == "list" {
        let list = response.success.unwrap_or_else(|| "".into());
        if opts.is_json_output {
            println!("{}", &list);
        } else {
            let apps_list: Vec<AppsObject> = serde_json::from_str(&list)?;
            let mut table = Table::new();
            table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
            // Create the header.
            table.set_titles(row![
                "Name",
                "Status",
                "Install State",
                "Manifest URL",
                "Update Sate",
                "Update URL"
            ]);
            // Add a row for each app.
            for app in apps_list {
                table.add_row(row![
                    app.name,
                    app.status,
                    app.install_state,
                    app.manifest_url,
                    app.update_state,
                    app.update_url
                ]);
            }
            table.printstd();
        }
    }

    Ok(())
}

fn build_install_request(
    maybe_device: Option<AdbDevice>,
    opts: &CmdOptions,
    path_param: &str,
) -> Result<Request, CmdLineError> {
    let path = Path::new(path_param);
    if !path.exists() {
        return Err(CmdLineError::Other(format!(
            "No such file or directory: {}",
            path_param
        )));
    }

    // Build the zip file if needed.
    let mut dir = env::temp_dir();
    dir.push("sideloaded_application.zip");
    let file_path = dir
        .into_os_string()
        .into_string()
        .map_err(|_| CmdLineError::Other("Failed creating temp file".into()))?;

    if path.is_dir() {
        create_zip_for_dir(path_param, &file_path)?;
    } else {
        // Do a copy to avoid touching original file
        std::fs::copy(path_param, &file_path)?;
    }

    // For socket connection
    if opts.socket_path.is_some() {
        return Ok(Request {
            cmd: "install".into(),
            param: Some(file_path),
        });
    }

    const DEST_PATH: &str = "/data/local/tmp/sideloaded_application.zip";

    if let Some(device) = maybe_device {
        // Push the file to the device.
        device.push(&file_path, DEST_PATH)?;
    } else {
        return Err(CmdLineError::Other("AdbDeviceError".into()));
    }

    Ok(Request {
        cmd: "install".into(),
        param: Some(DEST_PATH.into()),
    })
}

#[cfg(target_os = "windows")]
fn send(opts: &CmdOptions, request: &Request) -> Result<(), CmdLineError> {
    if opts.socket_path.is_some() {
        Err(CmdLineError::Other("--socket is not supported".into()))
    } else {
        // Forward localhost to device
        let stream = TcpStream::connect(&format!("localhost:{}", FORWARDING_PORT))?;
        send_request(&opts, &request, &stream)?;
        stream.shutdown(Shutdown::Both)?;
        Ok(())
    }
}

#[cfg(not(target_os = "windows"))]
fn send(opts: &CmdOptions, request: &Request) -> Result<(), CmdLineError> {
    if let Some(path) = &opts.socket_path {
        // Connect to uds locally
        let stream = UnixStream::connect(path)?;
        send_request(&opts, &request, &stream)?;
        stream.shutdown(Shutdown::Both)?;
    } else {
        // Forward localhost to device
        let stream = TcpStream::connect(&format!("localhost:{}", FORWARDING_PORT))?;
        send_request(&opts, &request, &stream)?;
        stream.shutdown(Shutdown::Both)?;
    }

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        error!("{}", err);
    }
}

fn run() -> Result<(), CmdLineError> {
    env_logger::init();

    let matches = App::new("appscmd")
        .version(crate_version!())
        .about("Manages apps installed on a b2g device, simulator or desktop.")
        .arg(
            Arg::with_name("json")
                .short("j")
                .long("json")
                .takes_value(false)
                .help("Set output as json format"),
        )
        .arg(
            Arg::with_name("socket")
                .long("socket")
                .takes_value(true)
                .help("Socket path to connect"),
        )
        .subcommand(
            SubCommand::with_name("install")
                .about("Install an application")
                .arg(
                    Arg::with_name("path")
                        .required(true)
                        .help("The path to the application's directory"),
                ),
        )
        .subcommand(
            SubCommand::with_name("choose-install-folder")
                .about("Install an application via file explorer"),
        )
        .subcommand(
            SubCommand::with_name("install-pwa")
                .about("Install a progressive web app")
                .arg(
                    Arg::with_name("url")
                        .required(true)
                        .help("The URL of the PWA to install"),
                ),
        )
        .subcommand(
            SubCommand::with_name("uninstall")
                .about("Uninstall an application")
                .arg(
                    Arg::with_name("manifest_url")
                        .required(true)
                        .help("The manifest URL of the application to uninstall"),
                ),
        )
        .subcommand(SubCommand::with_name("list").about("List installed applications"))
        .get_matches();

    let opts = CmdOptions {
        is_json_output: matches.is_present("json"),
        socket_path: matches.value_of("socket").map(|s| s.into()),
    };

    let maybe_device = if opts.socket_path.is_none() {
        // If no socket path is specified, create an adb based device using port forwarding
        // from tcp:6001 to the uds socket on device.
        Some(AdbDevice::new(FORWARDING_PORT, "/data/local/tmp/apps-uds.sock")?)
    } else {
        None
    };

    let request = if let Some(sub_command) = matches.subcommand_matches("install") {
        build_install_request(maybe_device, &opts, sub_command.value_of("path").unwrap())?
    } else if matches
        .subcommand_matches("choose-install-folder")
        .is_some()
    {
        let home_pathbuf =
            home_dir().ok_or_else(|| CmdLineError::Other("No home directory".into()))?;
        let home = home_pathbuf
            .to_str()
            .ok_or_else(|| CmdLineError::Other("Invalid home directory".into()))?;
        if let Some(path) = select_folder_dialog("Choose the app directory", home) {
            build_install_request(maybe_device, &opts, &path)?
        } else {
            return Err(CmdLineError::Other("No directory provided...".into()));
        }
    } else if let Some(sub_command) = matches.subcommand_matches("install-pwa") {
        Request {
            cmd: "install-pwa".into(),
            param: Some(sub_command.value_of("url").unwrap().into()),
        }
    } else if let Some(sub_command) = matches.subcommand_matches("uninstall") {
        Request {
            cmd: "uninstall".into(),
            param: Some(sub_command.value_of("manifest_url").unwrap().into()),
        }
    } else if matches.subcommand_matches("list").is_some() {
        Request {
            cmd: "list".into(),
            param: None,
        }
    } else {
        return Err(CmdLineError::Other("No valid command provided...".into()));
    };

    send(&opts, &request)
}
