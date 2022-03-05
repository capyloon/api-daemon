//! A minimal client for connecting to the tor network
//!
//! This crate is the primary command-line interface for
//! [Arti](https://gitlab.torproject.org/tpo/core/arti/), a project to
//! implement [Tor](https://www.torproject.org/) in Rust.
//! Many other crates in Arti depend on it.
//!
//! Note that Arti is a work in progress; although we've tried to
//! write all the critical security components, you probably shouldn't
//! use Arti in production until it's a bit more mature.
//!
//! More documentation will follow as this program improves.  For now,
//! just know that it can run as a simple SOCKS proxy over the Tor network.
//! It will listen on port 9150 by default, but you can override this in
//! the configuration.
//!
//! # Command-line interface
//!
//! (This is not stable; future versions will break this.)
//!
//! `arti` uses the [`clap`](https://docs.rs/clap/) crate for command-line
//! argument parsing; run `arti help` to get it to print its documentation.
//!
//! The only currently implemented subcommand is `arti proxy`; try
//! `arti help proxy` for a list of options you can pass to it.
//!
//! # Configuration
//!
//! By default, `arti` looks for its configuration files in a
//! platform-dependent location.  That's `~/.config/arti/arti.toml` on
//! Unix. (TODO document OSX and Windows.)
//!
//! The configuration file is TOML.  (We do not guarantee its stability.)
//! For an example see [`arti_defaults.toml`](./arti_defaults.toml).
//!
//! # Compile-time features
//!
//! `tokio` (default): Use the tokio runtime library as our backend.
//!
//! `async-std`: Use the async-std runtime library as our backend.
//! This feature has no effect unless building with `--no-default-features`
//! to disable tokio.

//! `native-tls` -- Build with support for the `native_tls` TLS
//! backend. (default)
//!
//! `rustls` -- Build with support for the `rustls` TLS backend.
//!
//! `static` -- Link with static versions of your system dependencies,
//! including sqlite and/or openssl.  (⚠ Warning ⚠: this feature will
//! include a dependency on native-tls, even if you weren't planning
//! to use native-tls.  If you only want to build with a static sqlite
//! library, enable the `static-sqlite` feature.  We'll look for
//! better solutions here in the future.)
//!
//! `static-sqlite` -- Link with a static version of sqlite.
//!
//! `static-native-tls` -- Link with a static version of `native-tls`.
//! Enables `native-tls`.
//!
//! # Limitations
//!
//! There are many missing features.  Among them: there's no onion
//! service support yet. There's no anti-censorship support.  You
//! can't be a relay.  There isn't any kind of proxy besides SOCKS.
//!
//! See the [README
//! file](https://gitlab.torproject.org/tpo/core/arti/-/blob/main/README.md)
//! for a more complete list of missing features.

#![warn(missing_docs)]
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
#![warn(clippy::needless_borrow)]
#![warn(clippy::needless_pass_by_value)]
#![warn(clippy::option_option)]
#![allow(clippy::print_stderr)] // Allowed in this crate only.
#![allow(clippy::print_stdout)] // Allowed in this crate only.
#![warn(clippy::rc_buffer)]
#![deny(clippy::ref_option_ref)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![warn(clippy::trait_duplication_in_bounds)]
#![deny(clippy::unnecessary_wraps)]
#![warn(clippy::unseparated_literal_suffix)]
#![deny(clippy::unwrap_used)]

use arti_client::{TorClient, TorClientConfig};
use tor_rtcompat::Runtime;

use anyhow::{Context, Result};
use futures::future::Abortable;
use tracing::{info, warn};

// Expose the proper runtime.
cfg_if::cfg_if! {
    if #[cfg(all(feature="tokio", feature="native-tls"))] {
        pub use tor_rtcompat::tokio::TokioNativeTlsRuntime as ChosenRuntime;
    } else if #[cfg(all(feature="tokio", feature="rustls"))] {
        pub use tor_rtcompat::tokio::TokioRustlsRuntime as ChosenRuntime;
    } else if #[cfg(all(feature="async-std", feature="native-tls"))] {
        pub use tor_rtcompat::tokio::TokioRustlsRuntime as ChosenRuntime;
    } else if #[cfg(all(feature="async-std", feature="rustls"))] {
        pub use tor_rtcompat::tokio::TokioRustlsRuntime as ChosenRuntime;
    }
}

/// Run the main loop of the proxy.
pub async fn run<R: Runtime>(
    runtime: R,
    socks_port: u16,
    config_sources: arti_config::ConfigurationSources,
    arti_config: arti_config::ArtiConfig,
    client_config: TorClientConfig,
) -> Result<()> {
    // Using OnDemand arranges that, while we are bootstrapping, incoming connections wait
    // for bootstrap to complete, rather than getting errors.
    use arti_client::BootstrapBehavior::OnDemand;
    use futures::FutureExt;
    let client = TorClient::with_runtime(runtime.clone())
        .config(client_config)
        .bootstrap_behavior(OnDemand)
        .create_unbootstrapped()?;

    if arti_config.application().watch_configuration() {
        crate::watch_cfg::watch_for_config_changes(config_sources, arti_config, client.clone())?;
    }

    futures::select!(
        r = crate::exit::wait_for_ctrl_c().fuse()
            => r.context("waiting for termination signal"),
        r = crate::proxy::run_socks_proxy(runtime, client.clone(), socks_port).fuse()
            => r.context("SOCKS proxy failure"),
        r = async {
            client.bootstrap().await?;
            info!("Sufficiently bootstrapped; system SOCKS now functional.");
            futures::future::pending::<Result<()>>().await
        }.fuse()
            => r.context("bootstrap"),
    )
}

/// Run the main loop of the proxy, letting the caller the possibility
/// to stop it using an abortable.
pub async fn run_abortable<F, R: Runtime, T: futures::Future<Output = ()>>(
    runtime: R,
    arti_config: arti_config::ArtiConfig,
    client_config: TorClientConfig,
    abortable: Abortable<T>,
    status_fn: F,
) -> Result<()>
where
    F: Fn(&arti_client::status::BootstrapStatus),
{
    use futures::stream::StreamExt;

    // Using OnDemand arranges that, while we are bootstrapping, incoming connections wait
    // for bootstrap to complete, rather than getting errors.
    use arti_client::BootstrapBehavior::OnDemand;
    use futures::FutureExt;
    let client = TorClient::with_runtime(runtime.clone())
        .config(client_config)
        .bootstrap_behavior(OnDemand)
        .create_unbootstrapped()?;
    let socks_port = arti_config.proxy().socks_port().unwrap_or(9150);
    let mut events = client.bootstrap_events();

    futures::select!(
        r = async {
            while let Some(status) = events.next().await {
                status_fn(&status);
            }
            futures::future::pending::<Result<()>>().await
        }.fuse() => r.context("events"),
        r = crate::proxy::run_socks_proxy(runtime, client.clone(), socks_port).fuse()
            => r.context("SOCKS proxy failure"),
        r = async {
            client.bootstrap().await?;
            info!("Sufficiently bootstrapped; system SOCKS now functional.");
            futures::future::pending::<Result<()>>().await
        }.fuse()
            => r.context("bootstrap"),
        r = abortable.fuse() => r.context("Aborted"),
    )
}
