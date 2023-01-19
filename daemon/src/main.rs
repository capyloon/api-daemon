#[cfg(feature = "breakpad")]
use breakpad_sys::{init_breakpad, write_minidump};
use std::env;

#[cfg(not(target_os = "android"))]
use env_logger::Builder;
#[cfg(not(target_os = "android"))]
use std::io::Write;

#[macro_use]
mod services_macro;
mod api_server;
mod cache_middleware;
mod config;
#[cfg(feature = "breakpad")]
mod crash_uploader;
mod global_context;
mod session;
mod session_counter;
mod shared_state;
mod uds_server;

use crate::config::Config;
use crate::global_context::GlobalContext;
use crate::session_counter::SessionKind;
use crate::shared_state::enabled_services;
use common::remote_services_registrar::RemoteServicesRegistrar;
use common::traits::SharedServiceState;
use log::{error, info};
use signal_hook::consts::signal::*;
use signal_hook::iterator::Signals;
use std::thread;
use vhost_server::config::VhostApi;

static VERSION: &str = include_str!("../../version.in");

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

    // Set the kaios.api-daemon.version property.
    if let Err(err) =
        android_utils::AndroidProperties::set("kaios.api-daemon.version", VERSION.trim())
    {
        error!(
            "Failed to set kaios.api-daemon.version to '{}' : {:?}",
            VERSION, err
        );
    }
}

#[cfg(not(target_os = "android"))]
fn init_logger(_verbose: bool) {
    Builder::from_default_env()
        .format(|buf, record| {
            let ts = buf.timestamp();
            match record.module_path() {
                Some(module_path) => {
                    writeln!(
                        buf,
                        "{} {:<5} {} {}",
                        ts,
                        record.level(),
                        module_path,
                        record.args()
                    )
                }
                None => {
                    writeln!(buf, "{} {:<5} {}", ts, record.level(), record.args())
                }
            }
        })
        .try_init()
        .expect("Failed to initialize logger.");
}

// Displays the latest git commit hash and build date if it's set properly by
// the build script.
fn display_build_info(config: &Config, registrar: &RemoteServicesRegistrar) {
    info!(
        "Starting api-daemon {} {} {}",
        VERSION.trim(),
        env!("VERGEN_SHA"),
        env!("VERGEN_COMMIT_DATE")
    );
    info!("Services: {:?}", enabled_services(config, registrar));
}

// Logs status information about the daemon.
fn log_daemon_status() {
    // Display the numbe of active sessions.
    info!(
        "Active sessions: websocket={}, uds={}",
        SessionKind::Ws.count(),
        SessionKind::Uds.count()
    );

    crate::shared_state::log_services_state();
}

// Installs a signal handler for SIGUSR1 and display information about the
// daemon state when the signal is handled.
fn install_signal_handler() {
    let mut signals = Signals::new([SIGUSR1]).expect("Failed to create SIGUSR1 signal handler");
    let _thread = thread::spawn(move || {
        for signal in &mut signals {
            match signal {
                SIGUSR1 => {
                    info!("SIGUSR1 signal received!");
                    log_daemon_status();
                }
                _ => unreachable!(),
            }
        }
    });
}

fn main() {
    #[cfg(feature = "daemon")]
    {
        use daemonize::Daemonize;
        let daemonize = Daemonize::new()
            .exit_action(|| println!("Executed before master process exits"))
            .privileged_action(|| "Executed before drop privileges");

        match daemonize.start() {
            Ok(_) => println!("Success, daemonized"),
            Err(e) => eprintln!("Error, {}", e),
        }
    }

    let config_path = env::args().nth(1).unwrap_or_else(|| "config.toml".into());

    if let Ok(config) = Config::from_file(&config_path) {
        init_logger(config.general.verbose_log);

        let registrar = RemoteServicesRegistrar::new(
            &config.general.remote_services_config,
            &config.general.remote_services_path,
        );

        display_build_info(&config, &registrar);

        let global_context = GlobalContext::new(&config);

        // Init breakpad
        #[cfg(feature = "breakpad")]
        {
            let mut log_path = config.general.log_path.clone();
            log_path.push_str("/api-daemon-crashes");
            info!("Saving mini dump into directory {}", log_path);
            let _ = std::fs::create_dir_all(&log_path);
            let exception_handler = init_breakpad(log_path.clone());
            // Write minidump while panic
            std::panic::set_hook(Box::new(move |_| {
                write_minidump(exception_handler);
            }));

            let uploader = crash_uploader::CrashUploader::new(&log_path);
            let can_upload = uploader.can_upload();
            info!("Will upload crash reports: {}", can_upload);
            if can_upload {
                uploader.upload_reports();
            } else {
                uploader.wipe_reports();
            }
        }

        install_signal_handler();

        // Start the vhost server
        #[cfg(feature = "virtual-host")]
        let vhost_data = vhost_server::vhost_data(&config.vhost);

        #[cfg(feature = "apps-service")]
        {
            let shared = apps_service::service::AppsService::shared_state();
            shared.lock().config.resolve_paths();
            #[cfg(feature = "virtual-host")]
            {
                shared.lock().vhost_api = VhostApi::new(vhost_data.clone());
            }
            apps_service::start_registry(shared, config.general.port);
        }

        #[cfg(feature = "procmanager-service")]
        {
            let shared = procmanager_service::service::ProcManagerService::shared_state();
            shared.lock().hints.config = config.procmanager_service;
            shared.lock().hints.after_config();
        }

        // Starts the web socket server in its own thread.
        let ws_context = global_context.clone();
        let actix_handle = thread::Builder::new()
            .name("actix ws server".into())
            .spawn(move || {
                #[cfg(feature = "device-telemetry")]
                api_server::start(&ws_context, vhost_data, telemetry_sender);

                #[cfg(not(feature = "device-telemetry"))]
                api_server::start(&ws_context, vhost_data, ());
            })
            .expect("Failed to start ws server thread");

        // Starts the unix domain socket server in its own thread.
        let uds_handle = thread::Builder::new()
            .name("uds server".into())
            .spawn(move || {
                #[cfg(feature = "device-telemetry")]
                uds_server::start(&global_context, telemetry);

                #[cfg(not(feature = "device-telemetry"))]
                uds_server::start(&global_context, ());
            })
            .expect("Failed to start uds server thread");

        uds_handle.join().expect("Failed to join the uds thread.");
        actix_handle
            .join()
            .expect("Failed to join the actix thread.");
    } else {
        init_logger(true);
        error!("Config file not found or invalid at {}", config_path);
    }
}
