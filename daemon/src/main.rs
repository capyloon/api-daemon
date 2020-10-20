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
mod config;
mod global_context;
mod session;
mod shared_state;
mod uds_server;

use crate::config::Config;
use crate::global_context::GlobalContext;
use crate::shared_state::{enabled_services, SharedStateKind};
use common::remote_services_registrar::RemoteServicesRegistrar;
use log::{error, info};
use std::thread;

static VERSION: &'static str = include_str!("../../version.in");

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
        // Init breakpad
        #[cfg(feature = "breakpad")]
        {
            use std::panic;
            let log_path = config.general.log_path.clone();
            info!("save mini dump into file: {}", log_path);
            let exception_handler = init_breakpad(log_path);
            // Write minidump while panic
            panic::set_hook(Box::new(move |_| {
                write_minidump(exception_handler);
            }));
        }

        let global_context = GlobalContext::new(&config);

        // Start the vhost server
        #[cfg(feature = "virtual-host")]
        let vhost_api = vhost_server::start_server(&config.vhost);

        #[cfg(feature = "apps-service")]
        {
            let service_state = global_context.service_state();
            let lock = service_state.lock();
            let shared_data = match lock.get(&"AppsManager".to_string()) {
                Some(SharedStateKind::AppsService(data)) => data,
                _ => panic!("Missing shared state for AppsService!!"),
            };
            let mut shared = shared_data.lock();
            shared.config = config.apps_service;
            #[cfg(feature = "virtual-host")]
            {
                shared.vhost_api = vhost_api;
            }
            apps_service::start_registry(
                shared_data.clone(),
                config.vhost.port
            );
        }

        // Starts the web socket server in its own thread.
        let ws_context = global_context.clone();
        let actix_handle = thread::Builder::new()
            .name("actix ws server".into())
            .spawn(move || {
                api_server::start(&ws_context);
            })
            .expect("Failed to start ws server thread");

        // Starts the unix domain socket server in its own thread.
        let uds_context = global_context.clone();
        let uds_handle = thread::Builder::new()
            .name("uds server".into())
            .spawn(move || {
                uds_server::start(&uds_context);
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
