//! Configure tracing subscribers for Arti

use anyhow::{anyhow, Context, Result};
use derive_builder::Builder;
use educe::Educe;
use fs_mistrust::Mistrust;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::str::FromStr;
use tor_config::impl_standard_builder;
use tor_config::{define_list_builder_accessors, define_list_builder_helper};
use tor_config::{CfgPath, ConfigBuildError};
use tracing::{warn, Subscriber};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{filter::Targets, fmt, registry, Layer};

/// Structure to hold our logging configuration options
#[derive(Debug, Clone, Builder, Eq, PartialEq)]
#[non_exhaustive] // TODO(nickm) remove public elements when I revise this.
#[builder(build_fn(error = "ConfigBuildError"))]
#[builder(derive(Debug, Serialize, Deserialize))]
pub struct LoggingConfig {
    /// Filtering directives that determine tracing levels as described at
    /// <https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/targets/struct.Targets.html#impl-FromStr>
    ///
    /// You can override this setting with the -l, --log-level command line parameter.
    ///
    /// Example: "info,tor_proto::channel=trace"
    #[builder(default = "default_console_filter()", setter(into, strip_option))]
    console: Option<String>,

    /// Filtering directives for the journald logger.
    ///
    /// Only takes effect if Arti is built with the `journald` filter.
    #[builder(
        setter(into),
        field(build = r#"tor_config::resolve_option(&self.journald, || None)"#)
    )]
    journald: Option<String>,

    /// Configuration for one or more logfiles.
    ///
    /// The default is not to log to any files.
    #[builder_field_attr(serde(default))]
    #[builder(sub_builder, setter(custom))]
    files: LogfileListConfig,

    /// If set to true, we disable safe logging on _all logs_, and store
    /// potentially sensitive information at level `info` or higher.
    ///
    /// This can be useful for debugging, but it increases the value of your
    /// logs to an attacker.  Do not turn this on in production unless you have
    /// a good log rotation mechanism.
    #[builder_field_attr(serde(default))]
    #[builder(default)]
    log_sensitive_information: bool,
}
impl_standard_builder! { LoggingConfig }

/// Return a default tracing filter value for `logging.console`.
#[allow(clippy::unnecessary_wraps)]
fn default_console_filter() -> Option<String> {
    Some("info".to_owned())
}

/// Local type alias, mostly helpful for derive_builder to DTRT
type LogfileListConfig = Vec<LogfileConfig>;

define_list_builder_helper! {
    struct LogfileListConfigBuilder {
        files: [LogfileConfigBuilder],
    }
    built: LogfileListConfig = files;
    default = vec![];
}

define_list_builder_accessors! {
    struct LoggingConfigBuilder {
        pub files: [LogfileConfigBuilder],
    }
}

/// Configuration information for an (optionally rotating) logfile.
#[derive(Debug, Builder, Clone, Eq, PartialEq)]
#[builder(derive(Debug, Serialize, Deserialize))]
#[builder(build_fn(error = "ConfigBuildError"))]
pub struct LogfileConfig {
    /// How often to rotate the file?
    #[builder(default)]
    rotate: LogRotation,
    /// Where to write the files?
    path: CfgPath,
    /// Filter to apply before writing
    filter: String,
}

impl_standard_builder! { LogfileConfig: !Default }

/// How often to rotate a log file
#[derive(Debug, Clone, Educe, Serialize, Deserialize, Copy, Eq, PartialEq)]
#[educe(Default)]
#[non_exhaustive]
#[serde(rename_all = "lowercase")]
pub enum LogRotation {
    /// Rotate logs daily
    Daily,
    /// Rotate logs hourly
    Hourly,
    /// Never rotate the log
    #[educe(Default)]
    Never,
}

/// As [`Targets::from_str`], but wrapped in an [`anyhow::Result`].
//
// (Note that we have to use `Targets`, not `EnvFilter`: see comment in
// `setup_logging()`.)
fn filt_from_str_verbose(s: &str, source: &str) -> Result<Targets> {
    Targets::from_str(s).with_context(|| format!("in {}", source))
}

/// As filt_from_str_verbose, but treat an absent filter (or an empty string) as
/// None.
fn filt_from_opt_str(s: &Option<String>, source: &str) -> Result<Option<Targets>> {
    Ok(match s {
        Some(s) if !s.is_empty() => Some(filt_from_str_verbose(s, source)?),
        _ => None,
    })
}

/// Helper to disable safe-logging when formatting an event to be logged to the
/// console.
struct FormatWithSafeLoggingSuppressed<F> {
    /// An inner formatting type that does the actual formatting.
    inner: F,
}
impl<S, N, F> fmt::FormatEvent<S, N> for FormatWithSafeLoggingSuppressed<F>
where
    F: fmt::FormatEvent<S, N>,
    N: for<'writer> fmt::FormatFields<'writer> + 'static,
    S: Subscriber + for<'span> tracing_subscriber::registry::LookupSpan<'span>,
{
    fn format_event(
        &self,
        ctx: &fmt::FmtContext<'_, S, N>,
        writer: fmt::format::Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        safelog::with_safe_logging_suppressed(|| self.inner.format_event(ctx, writer, event))
    }
}

/// Try to construct a tracing [`Layer`] for logging to stdout.
fn console_layer<S>(config: &LoggingConfig, cli: Option<&str>) -> Result<impl Layer<S>>
where
    S: Subscriber + for<'span> tracing_subscriber::registry::LookupSpan<'span>,
{
    let filter = cli
        .map(|s| filt_from_str_verbose(s, "--log-level command line parameter"))
        .or_else(|| filt_from_opt_str(&config.console, "logging.console").transpose())
        .unwrap_or_else(|| Ok(Targets::from_str("debug").expect("bad default")))?;
    // We suppress safe logging when formatting messages for the console,
    // which we assume to be volatile.
    let format = FormatWithSafeLoggingSuppressed {
        inner: fmt::format(),
    };
    Ok(fmt::Layer::default()
        .event_format(format)
        .with_filter(filter))
}

/// Try to construct a tracing [`Layer`] for logging to journald, if one is
/// configured.
#[cfg(feature = "journald")]
fn journald_layer<S>(config: &LoggingConfig) -> Result<impl Layer<S>>
where
    S: Subscriber + for<'span> tracing_subscriber::registry::LookupSpan<'span>,
{
    if let Some(filter) = filt_from_opt_str(&config.journald, "logging.journald")? {
        Ok(Some(tracing_journald::layer()?.with_filter(filter)))
    } else {
        // Fortunately, Option<Layer> implements Layer, so we can just return None here.
        Ok(None)
    }
}

/// Try to construct a non-blocking tracing [`Layer`] for writing data to an
/// optionally rotating logfile.
///
/// On success, return that layer, along with a WorkerGuard that needs to be
/// dropped when the program exits, to flush buffered messages.
fn logfile_layer<S>(
    config: &LogfileConfig,
    mistrust: &Mistrust,
) -> Result<(impl Layer<S> + Send + Sync + Sized, WorkerGuard)>
where
    S: Subscriber + for<'span> tracing_subscriber::registry::LookupSpan<'span> + Send + Sync,
{
    use tracing_appender::{
        non_blocking,
        rolling::{RollingFileAppender, Rotation},
    };

    let filter = filt_from_str_verbose(&config.filter, "logging.files.filter")?;
    let rotation = match config.rotate {
        LogRotation::Daily => Rotation::DAILY,
        LogRotation::Hourly => Rotation::HOURLY,
        _ => Rotation::NEVER,
    };
    let path = config.path.path()?;
    let directory = path.parent().unwrap_or_else(|| Path::new("."));
    mistrust.make_directory(directory)?;
    let fname = path
        .file_name()
        .ok_or_else(|| anyhow!("No path for log file"))
        .map(Path::new)?;

    let appender = RollingFileAppender::new(rotation, directory, fname);
    let (nonblocking, guard) = non_blocking(appender);
    let layer = fmt::layer().with_writer(nonblocking).with_filter(filter);
    Ok((layer, guard))
}

/// Try to construct a tracing [`Layer`] for all of the configured logfiles.
///
/// On success, return that layer along with a list of [`WorkerGuard`]s that
/// need to be dropped when the program exits.
fn logfile_layers<S>(
    config: &LoggingConfig,
    mistrust: &Mistrust,
) -> Result<(impl Layer<S>, Vec<WorkerGuard>)>
where
    S: Subscriber + for<'span> tracing_subscriber::registry::LookupSpan<'span> + Send + Sync,
{
    let mut guards = Vec::new();
    if config.files.is_empty() {
        // As above, we have Option<Layer> implements Layer, so we can return
        // None in this case.
        return Ok((None, guards));
    }

    let (layer, guard) = logfile_layer(&config.files[0], mistrust)?;
    guards.push(guard);

    // We have to use a dyn pointer here so we can build up linked list of
    // arbitrary depth.
    let mut layer: Box<dyn Layer<S> + Send + Sync + 'static> = Box::new(layer);

    for logfile in &config.files[1..] {
        let (new_layer, guard) = logfile_layer(logfile, mistrust)?;
        layer = Box::new(layer.and_then(new_layer));
        guards.push(guard);
    }

    Ok((Some(layer), guards))
}

/// Opaque structure that gets dropped when the program is shutting down,
/// after logs are no longer needed.  The `Drop` impl flushes buffered messages.
#[cfg_attr(feature = "experimental-api", visibility::make(pub))]
pub(crate) struct LogGuards {
    /// The actual list of guards we're returning.
    #[allow(unused)]
    guards: Vec<WorkerGuard>,

    /// A safelog guard, for use if we have decided to disable safe logging.
    #[allow(unused)]
    safelog_guard: Option<safelog::Guard>,
}

/// Set up logging.
///
/// Note that the returned LogGuard must be dropped precisely when the program
/// quits; they're used to ensure that all the log messages are flushed.
#[cfg_attr(feature = "experimental-api", visibility::make(pub))]
#[cfg_attr(docsrs, doc(cfg(feature = "experimental-api")))]
pub(crate) fn setup_logging(
    config: &LoggingConfig,
    mistrust: &Mistrust,
    cli: Option<&str>,
) -> Result<LogGuards> {
    // Important: We have to make sure that the individual layers we add here
    // are not filters themselves.  That means, for example, that we can't add
    // an `EnvFilter` layer unless we want it to apply globally to _all_ layers.
    //
    // For a bit of discussion on the difference between per-layer filters and filters
    // that apply to the entire registry, see
    // https://docs.rs/tracing-subscriber/0.3.5/tracing_subscriber/layer/index.html#global-filtering

    let registry = registry().with(console_layer(config, cli)?);

    #[cfg(feature = "journald")]
    let registry = registry.with(journald_layer(config)?);

    let (layer, guards) = logfile_layers(config, mistrust)?;
    let registry = registry.with(layer);

    registry.init();

    let safelog_guard = if config.log_sensitive_information {
        match safelog::disable_safe_logging() {
            Ok(guard) => Some(guard),
            Err(e) => {
                // We don't need to propagate this error; it isn't the end of
                // the world if we were unable to disable safe logging.
                warn!("Unable to disable safe logging: {}", e);
                None
            }
        }
    } else {
        None
    };

    Ok(LogGuards {
        guards,
        safelog_guard,
    })
}
