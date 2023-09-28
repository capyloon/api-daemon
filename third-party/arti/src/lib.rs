#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![doc = include_str!("../README.md")]
// @@ begin lint list maintained by maint/add_warning @@
#![cfg_attr(not(ci_arti_stable), allow(renamed_and_removed_lints))]
#![cfg_attr(not(ci_arti_nightly), allow(unknown_lints))]
#![deny(missing_docs)]
#![warn(noop_method_call)]
#![deny(unreachable_pub)]
#![warn(clippy::all)]
#![deny(clippy::await_holding_lock)]
#![deny(clippy::cargo_common_metadata)]
#![deny(clippy::cast_lossless)]
#![deny(clippy::checked_conversions)]
#![warn(clippy::cognitive_complexity)]
#![deny(clippy::debug_assert_with_mut_call)]
#![deny(clippy::exhaustive_enums)]
#![deny(clippy::exhaustive_structs)]
#![deny(clippy::expl_impl_clone_on_copy)]
#![deny(clippy::fallible_impl_from)]
#![deny(clippy::implicit_clone)]
#![deny(clippy::large_stack_arrays)]
#![warn(clippy::manual_ok_or)]
#![deny(clippy::missing_docs_in_private_items)]
#![deny(clippy::missing_panics_doc)]
#![warn(clippy::needless_borrow)]
#![warn(clippy::needless_pass_by_value)]
#![warn(clippy::option_option)]
#![deny(clippy::print_stderr)]
#![deny(clippy::print_stdout)]
#![warn(clippy::rc_buffer)]
#![deny(clippy::ref_option_ref)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![warn(clippy::trait_duplication_in_bounds)]
#![deny(clippy::unnecessary_wraps)]
#![warn(clippy::unseparated_literal_suffix)]
#![deny(clippy::unwrap_used)]
#![allow(clippy::let_unit_value)] // This can reasonably be done for explicitness
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::significant_drop_in_scrutinee)] // arti/-/merge_requests/588/#note_2812945
#![allow(clippy::result_large_err)] // temporary workaround for arti#587
//! <!-- @@ end lint list maintained by maint/add_warning @@ -->

// Overrides specific to this crate:
#![allow(clippy::print_stderr)]
#![allow(clippy::print_stdout)]

pub mod cfg;
pub mod logging;

#[cfg(all(feature = "experimental-api", feature = "dns-proxy"))]
pub mod dns;
#[cfg(feature = "experimental-api")]
pub mod exit;
#[cfg(feature = "experimental-api")]
pub mod process;
#[cfg(feature = "experimental-api")]
pub mod reload_cfg;
#[cfg(feature = "experimental-api")]
pub mod socks;

#[cfg(all(not(feature = "experimental-api"), feature = "dns-proxy"))]
mod dns;
#[cfg(not(feature = "experimental-api"))]
mod exit;
#[cfg(not(feature = "experimental-api"))]
mod process;
#[cfg(not(feature = "experimental-api"))]
mod reload_cfg;
#[cfg(not(feature = "experimental-api"))]
mod socks;

#[cfg(feature = "rpc")]
mod rpc;

use std::ffi::OsString;
use std::fmt::Write;

pub use cfg::{
    ApplicationConfig, ApplicationConfigBuilder, ArtiCombinedConfig, ArtiConfig, ArtiConfigBuilder,
    ProxyConfig, ProxyConfigBuilder, SystemConfig, SystemConfigBuilder, ARTI_EXAMPLE_CONFIG,
};
pub use logging::{LoggingConfig, LoggingConfigBuilder};

use arti_client::config::default_config_files;
use arti_client::{TorClient, TorClientConfig};
use safelog::with_safe_logging_suppressed;
use tor_config::ConfigurationSources;
use tor_rtcompat::{BlockOn, Runtime};

use anyhow::{Context, Error, Result};
use clap::{value_parser, Arg, ArgAction, Command};
#[allow(unused_imports)]
use tracing::{error, info, warn};

/// Shorthand for a boxed and pinned Future.
type PinnedFuture<T> = std::pin::Pin<Box<dyn futures::Future<Output = T>>>;

/// Create a runtime for Arti to use.
fn create_runtime() -> std::io::Result<impl Runtime> {
    cfg_if::cfg_if! {
        if #[cfg(feature="rpc")] {
            // TODO RPC: Because of
            // https://gitlab.torproject.org/tpo/core/arti/-/issues/837 , we can
            // currently define our RPC methods on TorClient<PreferredRuntime>.
            use tor_rtcompat::PreferredRuntime as ChosenRuntime;
        } else if #[cfg(all(feature="tokio", feature="native-tls"))] {
            use tor_rtcompat::tokio::TokioNativeTlsRuntime as ChosenRuntime;
        } else if #[cfg(all(feature="tokio", feature="rustls"))] {
            use tor_rtcompat::tokio::TokioRustlsRuntime as ChosenRuntime;
        } else if #[cfg(all(feature="async-std", feature="native-tls"))] {
            use tor_rtcompat::async_std::AsyncStdNativeTlsRuntime as ChosenRuntime;
        } else if #[cfg(all(feature="async-std", feature="rustls"))] {
            use tor_rtcompat::async_std::AsyncStdRustlsRuntime as ChosenRuntime;
        } else {
            compile_error!("You must configure both an async runtime and a TLS stack. See doc/TROUBLESHOOTING.md for more.");
        }
    }
    ChosenRuntime::create()
}

/// Return a (non-exhaustive) array of enabled Cargo features, for version printing purposes.
fn list_enabled_features() -> &'static [&'static str] {
    // HACK(eta): We can't get this directly, so we just do this awful hack instead.
    // Note that we only list features that aren't about the runtime used, since that already
    // gets printed separately.
    &[
        #[cfg(feature = "journald")]
        "journald",
        #[cfg(any(feature = "static-sqlite", feature = "static"))]
        "static-sqlite",
        #[cfg(any(feature = "static-native-tls", feature = "static"))]
        "static-native-tls",
    ]
}

/// Run the main loop of the proxy.
///
/// # Panics
///
/// Currently, might panic if things go badly enough wrong
#[cfg_attr(feature = "experimental-api", visibility::make(pub))]
#[cfg_attr(docsrs, doc(cfg(feature = "experimental-api")))]
async fn run<R: Runtime>(
    runtime: R,
    socks_port: u16,
    dns_port: u16,
    config_sources: ConfigurationSources,
    arti_config: ArtiConfig,
    client_config: TorClientConfig,
) -> Result<()> {
    // Using OnDemand arranges that, while we are bootstrapping, incoming connections wait
    // for bootstrap to complete, rather than getting errors.
    use arti_client::BootstrapBehavior::OnDemand;
    use futures::FutureExt;

    #[cfg(feature = "rpc")]
    let rpc_path = {
        if let Some(path) = &arti_config.rpc().rpc_listen {
            let path = path.path()?;
            let parent = path
                .parent()
                .ok_or(anyhow::anyhow!("No parent directory for rpc_listen path?"))?;
            client_config
                .fs_mistrust()
                .verifier()
                .make_secure_dir(parent)?;
            // It's just a unix thing; if we leave this sitting around, binding to it won't
            // work right.  There is probably a better solution.
            if path.exists() {
                std::fs::remove_file(&path)?;
            }

            Some(path)
        } else {
            None
        }
    };

    let client_builder = TorClient::with_runtime(runtime.clone())
        .config(client_config)
        .bootstrap_behavior(OnDemand);
    let client = client_builder.create_unbootstrapped()?;
    reload_cfg::watch_for_config_changes(config_sources, arti_config, client.clone())?;

    #[cfg(all(feature = "rpc", feature = "tokio"))]
    let rpc_mgr = {
        // TODO RPC This code doesn't really belong here; it's just an example.
        if let Some(listen_path) = rpc_path {
            // TODO Conceivably this listener belongs on a renamed "proxy" list.
            Some(rpc::launch_rpc_listener(
                &runtime,
                listen_path,
                client.clone(),
            )?)
        } else {
            None
        }
    };

    let mut proxy: Vec<PinnedFuture<(Result<()>, &str)>> = Vec::new();
    if socks_port != 0 {
        let runtime = runtime.clone();
        let client = client.isolated_client();
        proxy.push(Box::pin(async move {
            let res = socks::run_socks_proxy(
                runtime,
                client,
                socks_port,
                #[cfg(all(feature = "rpc", feature = "tokio"))]
                rpc_mgr,
            )
            .await;
            (res, "SOCKS")
        }));
    }

    #[cfg(feature = "dns-proxy")]
    if dns_port != 0 {
        let runtime = runtime.clone();
        let client = client.isolated_client();
        proxy.push(Box::pin(async move {
            let res = dns::run_dns_resolver(runtime, client, dns_port).await;
            (res, "DNS")
        }));
    }

    #[cfg(not(feature = "dns-proxy"))]
    if dns_port != 0 {
        warn!("Tried to specify a DNS proxy port, but Arti was built without dns-proxy support.");
        return Ok(());
    }

    if proxy.is_empty() {
        warn!("No proxy port set; specify -p PORT (for `socks_port`) or -d PORT (for `dns_port`). Alternatively, use the `socks_port` or `dns_port` configuration option.");
        return Ok(());
    }

    let proxy = futures::future::select_all(proxy).map(|(finished, _index, _others)| finished);
    futures::select!(
        r = exit::wait_for_ctrl_c().fuse()
            => r.context("waiting for termination signal"),
        r = proxy.fuse()
            => r.0.context(format!("{} proxy failure", r.1)),
        r = async {
            client.bootstrap().await?;
            info!("Sufficiently bootstrapped; system SOCKS now functional.");
            futures::future::pending::<Result<()>>().await
        }.fuse()
            => r.context("bootstrap"),
    )
}

/// Inner function, to handle a set of CLI arguments and return a single
/// `Result<()>` for convenient handling.
///
/// # ⚠️ Warning! ⚠️
///
/// If your program needs to call this function, you are setting yourself up for
/// some serious maintenance headaches.  See discussion on [`main`] and please
/// reach out to help us build you a better API.
///
/// # Panics
///
/// Currently, might panic if wrong arguments are specified.
#[cfg_attr(feature = "experimental-api", visibility::make(pub))]
fn main_main<I, T>(cli_args: I) -> Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    // We describe a default here, rather than using `default()`, because the
    // correct behavior is different depending on whether the filename is given
    // explicitly or not.
    let mut config_file_help = "Specify which config file(s) to read.".to_string();
    if let Ok(default) = default_config_files() {
        // If we couldn't resolve the default config file, then too bad.  If something
        // actually tries to use it, it will produce an error, but don't fail here
        // just for that reason.
        write!(config_file_help, " Defaults to {:?}", default).unwrap();
    }

    // We create the runtime now so that we can use its `Debug` impl to describe it for
    // the version string.
    let runtime = create_runtime()?;
    let features = list_enabled_features();
    let long_version = format!(
        "{}\nusing runtime: {:?}\noptional features: {}",
        env!("CARGO_PKG_VERSION"),
        runtime,
        if features.is_empty() {
            "<none>".into()
        } else {
            features.join(", ")
        }
    );

    let clap_app =
        Command::new("Arti")
            .version(env!("CARGO_PKG_VERSION"))
            .long_version(&long_version as &str)
            .author("The Tor Project Developers")
            .about("A Rust Tor implementation.")
            // HACK(eta): clap generates "arti [OPTIONS] <SUBCOMMAND>" for this usage string by
            //            default, but then fails to parse options properly if you do put them
            //            before the subcommand.
            //            We just declare all options as `global` and then require them to be
            //            put after the subcommand, hence this new usage string.
            .override_usage("arti <SUBCOMMAND> [OPTIONS]")
            .arg(
                Arg::new("config-files")
                    .short('c')
                    .long("config")
                    .action(ArgAction::Set)
                    .value_name("FILE")
                    .value_parser(value_parser!(OsString))
                    .action(ArgAction::Append)
                    // NOTE: don't forget the `global` flag on all arguments declared at this level!
                    .global(true)
                    .help(config_file_help.as_str()),
            )
            .arg(
                Arg::new("option")
                    .short('o')
                    .action(ArgAction::Set)
                    .value_name("KEY=VALUE")
                    .action(ArgAction::Append)
                    .global(true)
                    .help("Override config file parameters, using TOML-like syntax."),
            )
            .arg(
                Arg::new("loglevel")
                    .short('l')
                    .long("log-level")
                    .global(true)
                    .action(ArgAction::Set)
                    .value_name("LEVEL")
                    .help("Override the log level (usually one of 'trace', 'debug', 'info', 'warn', 'error')."),
            )
            .arg(
                Arg::new("disable-fs-permission-checks")
                    .long("disable-fs-permission-checks")
                    .global(true)
                    .action(ArgAction::SetTrue)
                    .help("Don't check permissions on the files we use."),
            )
            .subcommand(
                Command::new("proxy")
                    .about(
                        "Run Arti in SOCKS proxy mode, proxying connections through the Tor network.",
                    )
                    .arg(
                        Arg::new("socks-port")
                            .short('p')
                            .action(ArgAction::Set)
                            .value_name("PORT")
                            .help("Port to listen on for SOCKS connections (overrides the port in the config if specified).")
                    )
                    .arg(
                        Arg::new("dns-port")
                            .short('d')
                            .action(ArgAction::Set)
                            .value_name("PORT")
                            .help("Port to listen on for DNS request (overrides the port in the config if specified).")
                    )
            )
            .subcommand_required(true)
            .arg_required_else_help(true);

    // Tracing doesn't log anything when there is no subscriber set.  But we want to see
    // logging messages from config parsing etc.  We can't set the global default subscriber
    // because we can only set it once.  The other ways involve a closure.  So we have a
    // closure for all the startup code which runs *before* we set the logging properly.
    //
    // There is no cooked way to print our program name, so we do it like this.  This
    // closure is called to "make" a "Writer" for each message, so it runs at the right time:
    // before each message.
    let pre_config_logging_writer = || {
        // Weirdly, with .without_time(), tracing produces messages with a leading space.
        eprint!("arti:");
        std::io::stderr()
    };
    let pre_config_logging = tracing_subscriber::fmt()
        .without_time()
        .with_writer(pre_config_logging_writer)
        .finish();
    let pre_config_logging = tracing::Dispatch::new(pre_config_logging);
    let pre_config_logging_ret = tracing::dispatcher::with_default(&pre_config_logging, || {
        let matches = clap_app.try_get_matches_from(cli_args)?;

        let fs_mistrust_disabled = matches.get_flag("disable-fs-permission-checks");

        // A Mistrust object to use for loading our configuration.  Elsewhere, we
        // use the value _from_ the configuration.
        let cfg_mistrust = if fs_mistrust_disabled {
            fs_mistrust::Mistrust::new_dangerously_trust_everyone()
        } else {
            fs_mistrust::MistrustBuilder::default()
                .controlled_by_env_var(arti_client::config::FS_PERMISSIONS_CHECKS_DISABLE_VAR)
                .build()
                .expect("Could not construct default fs-mistrust")
        };

        let mut override_options: Vec<String> = matches
            .get_many::<String>("option")
            .unwrap_or_default()
            .cloned()
            .collect();
        if fs_mistrust_disabled {
            override_options.push("storage.permissions.dangerously_trust_everyone=true".to_owned());
        }

        let cfg_sources = {
            let mut cfg_sources = ConfigurationSources::from_cmdline(
                default_config_files()?,
                matches
                    .get_many::<OsString>("config-files")
                    .unwrap_or_default(),
                override_options,
            );
            cfg_sources.set_mistrust(cfg_mistrust);
            cfg_sources
        };

        let cfg = cfg_sources.load()?;
        let (config, client_config) =
            tor_config::resolve::<ArtiCombinedConfig>(cfg).context("read configuration")?;

        let log_mistrust = client_config.fs_mistrust().clone();

        Ok::<_, Error>((matches, cfg_sources, config, client_config, log_mistrust))
    })?;
    // Sadly I don't seem to be able to persuade rustfmt to format the two lists of
    // variable names identically.
    let (matches, cfg_sources, config, client_config, log_mistrust) = pre_config_logging_ret;

    let _log_guards = logging::setup_logging(
        config.logging(),
        &log_mistrust,
        matches.get_one::<String>("loglevel").map(|s| s.as_str()),
    )?;

    if !config.application().allow_running_as_root {
        process::exit_if_root();
    }

    #[cfg(feature = "harden")]
    if !config.application().permit_debugging {
        if let Err(e) = process::enable_process_hardening() {
            error!("Encountered a problem while enabling hardening. To disable this feature, set application.permit_debugging to true.");
            return Err(e);
        }
    }

    if let Some(proxy_matches) = matches.subcommand_matches("proxy") {
        let socks_port = match (
            proxy_matches.get_one::<String>("socks-port"),
            config.proxy().socks_listen.localhost_port_legacy()?,
        ) {
            (Some(p), _) => p.parse().expect("Invalid port specified"),
            (None, Some(s)) => s,
            (None, None) => 0,
        };

        let dns_port = match (
            proxy_matches.get_one::<String>("dns-port"),
            config.proxy().dns_listen.localhost_port_legacy()?,
        ) {
            (Some(p), _) => p.parse().expect("Invalid port specified"),
            (None, Some(s)) => s,
            (None, None) => 0,
        };

        info!(
            "Starting Arti {} in SOCKS proxy mode on port {}...",
            env!("CARGO_PKG_VERSION"),
            socks_port
        );

        process::use_max_file_limit(&config);

        let rt_copy = runtime.clone();
        rt_copy.block_on(run(
            runtime,
            socks_port,
            dns_port,
            cfg_sources,
            config,
            client_config,
        ))?;
        Ok(())
    } else {
        panic!("Subcommand added to clap subcommand list, but not yet implemented")
    }
}

/// Main program, callable directly from a binary crate's `main`
///
/// This function behaves the same as `main_main()`, except:
///   * It takes command-line arguments from `std::env::args_os` rather than
///     from an argument.
///   * It exits the process with an appropriate error code on error.
///
/// # ⚠️ Warning ⚠️
///
/// Calling this function, or the related experimental function `main_main`, is
/// probably a bad idea for your code.  It means that you are invoking Arti as
/// if from the command line, but keeping it embedded inside your process. Doing
/// this will block your process take over handling for several signal types,
/// possibly disable debugger attachment, and a lot more junk that a library
/// really has no business doing for you.  It is not designed to run in this
/// way, and may give you strange results.
///
/// If the functionality you want is available in [`arti_client`] crate, or from
/// a *non*-experimental API in this crate, it would be better for you to use
/// that API instead.
///
/// Alternatively, if you _do_ need some underlying function from the `arti`
/// crate, it would be better for all of us if you had a stable interface to that
/// function. Please reach out to the Arti developers, so we can work together
/// to get you the stable API you need.
pub fn main() {
    match main_main(std::env::args_os()) {
        Ok(()) => {}
        Err(e) => {
            if let Some(arti_err) = e.downcast_ref::<arti_client::Error>() {
                if let Some(hint) = arti_err.hint() {
                    info!("{}", hint);
                }
            }
            match e.downcast_ref::<clap::Error>() {
                Some(clap_err) => clap_err.exit(),
                None => with_safe_logging_suppressed(|| tor_error::report_and_exit(e)),
            }
        }
    }
}
