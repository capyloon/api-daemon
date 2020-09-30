use common::expose_remote_service;
use log::{error, info};
use std::io::Write;

#[cfg(target_os = "android")]
fn init_logger(verbose: bool) {
    use android_logger::Filter;
    use log::Level;

    let level = if verbose {
        Filter::default().with_min_level(Level::Debug)
    } else {
        Filter::default().with_min_level(Level::Info)
    };
    android_logger::init_once(level);
}

#[cfg(not(target_os = "android"))]
fn init_logger(_verbose: bool) {
    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            let ts = buf.timestamp();
            match record.module_path() {
                Some(module_path) => {
                    return writeln!(
                        buf,
                        "{} {:<5} {} {}",
                        ts,
                        record.level(),
                        module_path,
                        record.args()
                    );
                }
                None => {
                    return writeln!(buf, "{} {:<5} {}", ts, record.level(), record.args());
                }
            }
        })
        .try_init()
        .expect("Failed to initialize logger.");
}

// Exposes a single service as remoted in this process.
// This generates a `session` module in which a Session struct is public
// and will manage the whole cycle once calling `Session::start()`
expose_remote_service!(test_service::service::TestServiceImpl, test_service, TestServiceImpl);

fn main() {
    init_logger(true);

    info!("Starting child-test-daemon");

    match session::Session::start() {
        Err(session::SessionError::MissingEnvVar) => {
            error!("No fd specified, stopping.");
        }
        Err(session::SessionError::BadFdValue(fd_s)) => {
            error!("{} can't be used as a file descriptor, stopping", fd_s);
        }
        _ => {}
    }

    info!("child-test-daemon exiting");
}
