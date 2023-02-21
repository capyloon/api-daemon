//! Configuration for the Arti command line application
//
// (Thia module is called `cfg` to avoid name clash with the `config` crate, which we use.)

use paste::paste;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use arti_client::TorClientConfig;
use tor_config::resolve_alternative_specs;
pub(crate) use tor_config::{impl_standard_builder, ConfigBuildError, Listen};

use crate::{LoggingConfig, LoggingConfigBuilder};

/// Example file demonstrating our our configuration and the default options.
///
/// The options in this example file are all commented out;
/// the actual defaults are done via builder attributes in all the Rust config structs.
pub const ARTI_EXAMPLE_CONFIG: &str = concat!(include_str!("./arti-example-config.toml"),);

/// Test case file for the oldest version of the config we still support.
///
/// (When updating, copy `arti-example-config.toml` from the earliest version we want to
/// be compatible with.)
//
// Probably, in the long run, we will want to make this architecture more general: we'll want
// to have a larger number of examples to test, and we won't want to write a separate constant
// for each. Probably in that case, we'll want a directory of test examples, and we'll want to
// traverse the whole directory.
//
// Compare C tor, look at conf_examples and conf_failures - each of the subdirectories there is
// an example configuration situation that we wanted to validate.
//
// NB here in Arti the OLDEST_SUPPORTED_CONFIG and the ARTI_EXAMPLE_CONFIG are tested
// somewhat differently: we test that the current example is *exhaustive*, not just
// parseable.
#[cfg(test)]
const OLDEST_SUPPORTED_CONFIG: &str = concat!(include_str!("./oldest-supported-config.toml"),);

/// Structure to hold our application configuration options
#[derive(Debug, Clone, Builder, Eq, PartialEq)]
#[builder(build_fn(error = "ConfigBuildError"))]
#[builder(derive(Debug, Serialize, Deserialize))]
pub struct ApplicationConfig {
    /// If true, we should watch our configuration files for changes, and reload
    /// our configuration when they change.
    ///
    /// Note that this feature may behave in unexpected ways if the path to the
    /// directory holding our configuration files changes its identity (because
    /// an intermediate symlink is changed, because the directory is removed and
    /// recreated, or for some other reason).
    #[builder(default)]
    pub(crate) watch_configuration: bool,

    /// If true, we should allow other applications not owned by the system
    /// administrator to monitor the Arti application and inspect its memory.
    ///
    /// Otherwise, we take various steps (including disabling core dumps) to
    /// make it harder for other programs to view our internal state.
    ///
    /// This option has no effect when arti is built without the `harden`
    /// feature.  When `harden` is not enabled, debugger attachment is permitted
    /// whether this option is set or not.
    #[builder(default)]
    pub(crate) permit_debugging: bool,

    /// If true, then we do not exit when we are running as `root`.
    ///
    /// This has no effect on Windows.
    #[builder(default)]
    pub(crate) allow_running_as_root: bool,
}
impl_standard_builder! { ApplicationConfig }

/// Resolves values from `$field_listen` and `$field_port` (compat) into a `Listen`
///
/// For `dns` and `proxy`.
///
/// Handles defaulting, and normalisation, using `resolve_alternative_specs`
/// and `Listen::new_localhost_option`.
///
/// Broken out into a macro so as to avoid having to state the field name four times,
/// which is a recipe for programming slips.
macro_rules! resolve_listen_port {
    { $self:expr, $field:ident, $def_port:expr } => { paste!{
        resolve_alternative_specs(
            [
                (
                    concat!(stringify!($field), "_listen"),
                    $self.[<$field _listen>].clone(),
                ),
                (
                    concat!(stringify!($field), "_port"),
                    $self.[<$field _port>].map(Listen::new_localhost_optional),
                ),
            ],
            || Listen::new_localhost($def_port),
        )?
    } }
}

/// Configuration for one or more proxy listeners.
#[derive(Debug, Clone, Builder, Eq, PartialEq)]
#[builder(build_fn(error = "ConfigBuildError"))]
#[builder(derive(Debug, Serialize, Deserialize))]
#[allow(clippy::option_option)] // Builder port fields: Some(None) = specified to disable
pub struct ProxyConfig {
    /// Addresses to listen on for incoming SOCKS connections.
    #[builder(field(build = r#"resolve_listen_port!(self, socks, 9150)"#))]
    pub(crate) socks_listen: Listen,

    /// Port to listen on (at localhost) for incoming SOCKS connections.
    ///
    /// This field is deprecated, and will, eventually, be removed.
    /// Use `socks_listen` instead, which accepts the same values,
    /// but which will also be able to support more flexible listening in the future.
    #[builder(
        setter(strip_option),
        field(type = "Option<Option<u16>>", build = "()")
    )]
    #[builder_setter_attr(deprecated)]
    pub(crate) socks_port: (),

    /// Addresses to listen on for incoming DNS connections.
    #[builder(field(build = r#"resolve_listen_port!(self, dns, 0)"#))]
    pub(crate) dns_listen: Listen,

    /// Port to lisen on (at localhost) for incoming DNS connections.
    ///
    /// This field is deprecated, and will, eventually, be removed.
    /// Use `dns_listen` instead, which accepts the same values,
    /// but which will also be able to support more flexible listening in the future.
    #[builder(
        setter(strip_option),
        field(type = "Option<Option<u16>>", build = "()")
    )]
    #[builder_setter_attr(deprecated)]
    pub(crate) dns_port: (),
}
impl_standard_builder! { ProxyConfig }

/// Configuration for system resources used by Tor.
///
/// You cannot change this section on a running Arti client.
#[derive(Debug, Clone, Builder, Eq, PartialEq)]
#[builder(build_fn(error = "ConfigBuildError"))]
#[builder(derive(Debug, Serialize, Deserialize))]
#[non_exhaustive]
pub struct SystemConfig {
    /// Maximum number of file descriptors we should launch with
    #[builder(setter(into), default = "default_max_files()")]
    pub(crate) max_files: u64,
}
impl_standard_builder! { SystemConfig }

/// Return the default maximum number of file descriptors to launch with.
fn default_max_files() -> u64 {
    16384
}

/// Structure to hold Arti's configuration options, whether from a
/// configuration file or the command line.
//
/// These options are declared in a public crate outside of `arti` so that other
/// applications can parse and use them, if desired.  If you're only embedding
/// arti via `arti-client`, and you don't want to use Arti's configuration
/// format, use [`arti_client::TorClientConfig`] instead.
///
/// By default, Arti will run using the default Tor network, store state and
/// cache information to a per-user set of directories shared by all
/// that user's applications, and run a SOCKS client on a local port.
///
/// NOTE: These are NOT the final options or their final layout. Expect NO
/// stability here.
#[derive(Debug, Builder, Clone, Eq, PartialEq)]
#[builder(derive(Serialize, Deserialize, Debug))]
#[builder(build_fn(error = "ConfigBuildError"))]
pub struct ArtiConfig {
    /// Configuration for application behavior.
    #[builder(sub_builder)]
    #[builder_field_attr(serde(default))]
    application: ApplicationConfig,

    /// Configuration for proxy listeners
    #[builder(sub_builder)]
    #[builder_field_attr(serde(default))]
    proxy: ProxyConfig,

    /// Logging configuration
    #[builder(sub_builder)]
    #[builder_field_attr(serde(default))]
    logging: LoggingConfig,

    /// Information on system resources used by Arti.
    #[builder(sub_builder)]
    #[builder_field_attr(serde(default))]
    pub(crate) system: SystemConfig,
}
impl_standard_builder! { ArtiConfig }

impl tor_config::load::TopLevel for ArtiConfig {
    type Builder = ArtiConfigBuilder;
    const DEPRECATED_KEYS: &'static [&'static str] = &["proxy.socks_port", "proxy.dns_port"];
}

/// Convenience alias for the config for a whole `arti` program
///
/// Used primarily as a type parameter on calls to [`tor_config::resolve`]
pub type ArtiCombinedConfig = (ArtiConfig, TorClientConfig);

impl ArtiConfig {
    /// Return the [`ApplicationConfig`] for this configuration.
    pub fn application(&self) -> &ApplicationConfig {
        &self.application
    }

    /// Return the [`LoggingConfig`] for this configuration.
    pub fn logging(&self) -> &LoggingConfig {
        &self.logging
    }

    /// Return the [`ProxyConfig`] for this configuration.
    pub fn proxy(&self) -> &ProxyConfig {
        &self.proxy
    }
}

#[cfg(test)]
mod test {
    // @@ begin test lint list maintained by maint/add_warning @@
    #![allow(clippy::bool_assert_comparison)]
    #![allow(clippy::clone_on_copy)]
    #![allow(clippy::dbg_macro)]
    #![allow(clippy::print_stderr)]
    #![allow(clippy::print_stdout)]
    #![allow(clippy::unwrap_used)]
    //! <!-- @@ end test lint list maintained by maint/add_warning @@ -->

    use arti_client::config::dir;
    use arti_client::config::TorClientConfigBuilder;
    use itertools::{chain, Itertools};
    use regex::Regex;
    use std::iter;
    use std::time::Duration;
    use tor_config::load::ResolutionResults;

    use super::*;

    fn uncomment_example_settings(template: &str) -> String {
        let re = Regex::new(r#"(?m)^\#([^ \n])"#).unwrap();
        re.replace_all(template, |cap: &regex::Captures<'_>| -> _ {
            cap.get(1).unwrap().as_str().to_string()
        })
        .into()
    }

    #[test]
    fn default_config() {
        // See comment for OLDEST_SUPPORTED_CONFIG for likely future evolution
        let empty_config = config::Config::builder().build().unwrap();
        let empty_config: ArtiCombinedConfig = tor_config::resolve(empty_config).unwrap();

        let default = (ArtiConfig::default(), TorClientConfig::default());

        let parses_to_defaults = |example: &str, known_unrecognized_options: &[&str]| {
            let cfg = config::Config::builder()
                .add_source(config::File::from_str(example, config::FileFormat::Toml))
                .build()
                .unwrap();

            // This tests that the example settings do not *contradict* the defaults.
            //
            // Also we should ideally test that every setting from the config appears here in
            // the file.  Possibly that could be done with some kind of stunt Deserializer,
            // but it's not trivial.
            let results: ResolutionResults<ArtiCombinedConfig> =
                tor_config::resolve_return_results(cfg).unwrap();

            assert_eq!(&results.value, &default);
            assert_eq!(&results.value, &empty_config);

            // We serialize the DisfavouredKey entries to strings to compare them against
            // `known_unrecognized_options`.
            let unrecognized = results
                .unrecognized
                .iter()
                .map(|k| k.to_string())
                .collect_vec();

            assert_eq!(&unrecognized, &known_unrecognized_options);

            results.value
        };

        let _ = parses_to_defaults(ARTI_EXAMPLE_CONFIG, &[]);
        let _ = parses_to_defaults(OLDEST_SUPPORTED_CONFIG, &[]);

        #[allow(unused_mut)]
        let mut known_unrecognized_options = vec![];

        #[cfg(target_family = "windows")]
        known_unrecognized_options.extend([
            "storage.permissions.trust_group",
            "storage.permissions.trust_user",
        ]);

        let parsed = parses_to_defaults(
            &uncomment_example_settings(ARTI_EXAMPLE_CONFIG),
            &known_unrecognized_options,
        );
        let parsed_old = parses_to_defaults(
            &uncomment_example_settings(OLDEST_SUPPORTED_CONFIG),
            &known_unrecognized_options,
        );

        let built_default = (
            ArtiConfigBuilder::default().build().unwrap(),
            TorClientConfigBuilder::default().build().unwrap(),
        );
        assert_eq!(&parsed, &built_default);
        assert_eq!(&parsed_old, &built_default);
        assert_eq!(&default, &built_default);
    }

    #[test]
    fn builder() {
        use tor_config::CfgPath;
        let sec = std::time::Duration::from_secs(1);

        let auth = dir::Authority::builder()
            .name("Fred")
            .v3ident([22; 20].into())
            .clone();
        let mut fallback = dir::FallbackDir::builder();
        fallback
            .rsa_identity([23; 20].into())
            .ed_identity([99; 32].into())
            .orports()
            .push("127.0.0.7:7".parse().unwrap());

        let mut bld = ArtiConfig::builder();
        let mut bld_tor = TorClientConfig::builder();

        bld.proxy().socks_listen(Listen::new_localhost(9999));
        bld.logging().console("warn");

        bld_tor.tor_network().set_authorities(vec![auth]);
        bld_tor.tor_network().set_fallback_caches(vec![fallback]);
        bld_tor
            .storage()
            .cache_dir(CfgPath::new("/var/tmp/foo".to_owned()))
            .state_dir(CfgPath::new("/var/tmp/bar".to_owned()));
        bld_tor.download_schedule().retry_certs().attempts(10);
        bld_tor.download_schedule().retry_certs().initial_delay(sec);
        bld_tor.download_schedule().retry_certs().parallelism(3);
        bld_tor.download_schedule().retry_microdescs().attempts(30);
        bld_tor
            .download_schedule()
            .retry_microdescs()
            .initial_delay(10 * sec);
        bld_tor
            .download_schedule()
            .retry_microdescs()
            .parallelism(9);
        bld_tor
            .override_net_params()
            .insert("wombats-per-quokka".to_owned(), 7);
        bld_tor
            .path_rules()
            .ipv4_subnet_family_prefix(20)
            .ipv6_subnet_family_prefix(48);
        bld_tor.preemptive_circuits().disable_at_threshold(12);
        bld_tor
            .preemptive_circuits()
            .set_initial_predicted_ports(vec![80, 443]);
        bld_tor
            .preemptive_circuits()
            .prediction_lifetime(Duration::from_secs(3600))
            .min_exit_circs_for_port(2);
        bld_tor
            .circuit_timing()
            .max_dirtiness(90 * sec)
            .request_timeout(10 * sec)
            .request_max_retries(22)
            .request_loyalty(3600 * sec);
        bld_tor.address_filter().allow_local_addrs(true);

        let val = bld.build().unwrap();

        assert_ne!(val, ArtiConfig::default());
    }

    #[test]
    fn articonfig_application() {
        let config = ArtiConfig::default();

        let application = config.application();
        assert_eq!(&config.application, application);
    }

    #[test]
    fn articonfig_logging() {
        let config = ArtiConfig::default();

        let logging = config.logging();
        assert_eq!(&config.logging, logging);
    }

    #[test]
    fn articonfig_proxy() {
        let config = ArtiConfig::default();

        let proxy = config.proxy();
        assert_eq!(&config.proxy, proxy);
    }

    /// Comprehensive tests for the various `socks_port` and `dns_port`
    ///
    /// The "this isn't set at all, just use the default" cases are tested elsewhere.
    fn compat_ports_listen(
        f: &str,
        get_listen: &dyn Fn(&ArtiConfig) -> &Listen,
        bld_get_port: &dyn Fn(&ArtiConfigBuilder) -> &Option<Option<u16>>,
        bld_get_listen: &dyn Fn(&ArtiConfigBuilder) -> &Option<Listen>,
        setter_port: &dyn Fn(&mut ArtiConfigBuilder, Option<u16>) -> &mut ProxyConfigBuilder,
        setter_listen: &dyn Fn(&mut ArtiConfigBuilder, Listen) -> &mut ProxyConfigBuilder,
    ) {
        let from_toml = |s: &str| -> ArtiConfigBuilder {
            let cfg: toml::Value = toml::from_str(dbg!(s)).unwrap();
            let cfg: ArtiConfigBuilder = cfg.try_into().unwrap();
            cfg
        };

        let conflicting_cfgs = [
            format!("proxy.{}_port = 0 \n proxy.{}_listen = 200", f, f),
            format!("proxy.{}_port = 100 \n proxy.{}_listen = 0", f, f),
            format!("proxy.{}_port = 100 \n proxy.{}_listen = 200", f, f),
        ];

        let chk = |cfg: &ArtiConfigBuilder, expected: &Listen| {
            dbg!(bld_get_listen(cfg), bld_get_port(cfg));
            let cfg = cfg.build().unwrap();
            assert_eq!(get_listen(&cfg), expected);
        };

        let check_setters = |port, expected: &_| {
            for cfg in chain!(
                iter::once(ArtiConfig::builder()),
                conflicting_cfgs.iter().map(|cfg| from_toml(cfg)),
            ) {
                for listen in match port {
                    None => vec![Listen::new_none(), Listen::new_localhost(0)],
                    Some(port) => vec![Listen::new_localhost(port)],
                } {
                    let mut cfg = cfg.clone();
                    setter_port(&mut cfg, dbg!(port));
                    setter_listen(&mut cfg, dbg!(listen));
                    chk(&cfg, expected);
                }
            }
        };

        {
            let expected = Listen::new_localhost(100);

            let cfg = from_toml(&format!("proxy.{}_port = 100", f));
            assert_eq!(bld_get_port(&cfg), &Some(Some(100)));
            chk(&cfg, &expected);

            let cfg = from_toml(&format!("proxy.{}_listen = 100", f));
            assert_eq!(bld_get_listen(&cfg), &Some(Listen::new_localhost(100)));
            chk(&cfg, &expected);

            let cfg = from_toml(&format!(
                "proxy.{}_port = 100\n proxy.{}_listen = 100",
                f, f
            ));
            chk(&cfg, &expected);

            check_setters(Some(100), &expected);
        }

        {
            let expected = Listen::new_none();

            let cfg = from_toml(&format!("proxy.{}_port = 0", f));
            chk(&cfg, &expected);

            let cfg = from_toml(&format!("proxy.{}_listen = 0", f));
            chk(&cfg, &expected);

            let cfg = from_toml(&format!("proxy.{}_port = 0 \n proxy.{}_listen = 0", f, f));
            chk(&cfg, &expected);

            check_setters(None, &expected);
        }

        for cfg in &conflicting_cfgs {
            let cfg = from_toml(cfg);
            let err = dbg!(cfg.build()).unwrap_err();
            assert!(err.to_string().contains("specifying different values"));
        }
    }

    #[test]
    #[allow(deprecated)]
    fn ports_listen_socks() {
        compat_ports_listen(
            "socks",
            &|cfg| &cfg.proxy.socks_listen,
            &|bld| &bld.proxy.socks_port,
            &|bld| &bld.proxy.socks_listen,
            &|bld, arg| bld.proxy.socks_port(arg),
            &|bld, arg| bld.proxy.socks_listen(arg),
        );
    }

    #[test]
    #[allow(deprecated)]
    fn compat_ports_listen_dns() {
        compat_ports_listen(
            "dns",
            &|cfg| &cfg.proxy.dns_listen,
            &|bld| &bld.proxy.dns_port,
            &|bld| &bld.proxy.dns_listen,
            &|bld, arg| bld.proxy.dns_port(arg),
            &|bld, arg| bld.proxy.dns_listen(arg),
        );
    }

    #[allow(clippy::dbg_macro)]
    fn exhaustive_1(example_file: &str, expect_missing: &[&str]) {
        use serde_json::Value as JsValue;
        use std::collections::BTreeSet;

        let example = uncomment_example_settings(example_file);
        let example: toml::Value = toml::from_str(&example).unwrap();
        // dbg!(&example);
        let example = serde_json::to_value(&example).unwrap();
        // dbg!(&example);

        // "Exhaustive" taxonomy of the recognised configuration keys
        //
        // We use the JSON serialization of the default builders, because Rust's toml
        // implementation likes to omit more things, that we want to see.
        //
        // I'm not sure this is quite perfect but it is pretty good,
        // and has found a number of un-exampled config keys.
        let exhausts = [
            serde_json::to_value(&TorClientConfig::builder()).unwrap(),
            serde_json::to_value(&ArtiConfig::builder()).unwrap(),
        ];

        #[derive(Default, Debug)]
        struct Walk {
            current_path: Vec<String>,
            problems: Vec<(String, String)>,
        }

        impl Walk {
            /// Records a problem
            fn bad(&mut self, m: &str) {
                self.problems
                    .push((self.current_path.join("."), m.to_string()));
            }

            /// Recurses, looking for problems
            ///
            /// Visited for every node in either or both of the starting `exhausts`.
            ///
            /// `E` is the number of elements in `exhausts`, ie the number of different
            /// top-level config types that Arti uses.  Ie, 2.
            fn walk<const E: usize>(
                &mut self,
                example: Option<&JsValue>,
                exhausts: [Option<&JsValue>; E],
            ) {
                assert! { exhausts.into_iter().any(|e| e.is_some()) }

                let example = if let Some(e) = example {
                    e
                } else {
                    self.bad("missing from example");
                    return;
                };

                let tables = exhausts.map(|e| e?.as_object());

                // Union of the keys of both exhausts' tables (insofar as they *are* tables)
                let table_keys = tables
                    .iter()
                    .flat_map(|t| t.map(|t| t.keys().cloned()).into_iter().flatten())
                    .collect::<BTreeSet<String>>();

                for key in table_keys {
                    let example = if let Some(e) = example.as_object() {
                        e
                    } else {
                        // At least one of the exhausts was a nonempty table,
                        // but the corresponding example node isn't a table.
                        self.bad("expected table in example");
                        continue;
                    };

                    // Descend the same key in all the places.
                    self.current_path.push(key.clone());
                    self.walk(example.get(&key), tables.map(|t| t?.get(&key)));
                    self.current_path.pop().unwrap();
                }
            }
        }

        let exhausts = exhausts.iter().map(Some).collect_vec().try_into().unwrap();

        let mut walk = Walk::default();
        walk.walk::<2>(Some(&example), exhausts);
        let mut problems = walk.problems;

        // When adding things here, check that `arti-example-config.toml`
        // actually has something about these particular config keys.
        dbg!(&expect_missing);
        let expect_missing: Vec<&str> = ["tor_network.authorities", "tor_network.fallback_caches"]
            .into_iter()
            .chain(expect_missing.iter().cloned())
            .collect_vec();
        dbg!(&expect_missing);

        for exp in expect_missing {
            let was = problems.len();
            problems.retain(|(path, _)| path != exp);
            if problems.len() == was {
                problems.push((
                    exp.into(),
                    "expected to be missing but found in default".into(),
                ));
            }
        }

        let problems = problems
            .into_iter()
            .map(|(path, m)| format!("    config key {:?}: {}", path, m))
            .collect_vec();

        assert! { problems.is_empty(),
        "example config exhaustiveness check failed for {:?}:\n{}\n",
        example_file, problems.join("\n")}
    }

    #[test]
    fn exhaustive() {
        let mut deprecated = vec![];
        <(ArtiConfig, TorClientConfig) as tor_config::load::Resolvable>::enumerate_deprecated_keys(
            &mut |l| {
                for k in l {
                    deprecated.push(k.to_string());
                }
            },
        );
        let deprecated = deprecated.iter().map(|s| &**s).collect_vec();

        exhaustive_1(ARTI_EXAMPLE_CONFIG, &deprecated);

        exhaustive_1(
            OLDEST_SUPPORTED_CONFIG,
            // add *new*, not present in old file, settings here
            &[
                "application.allow_running_as_root",
                "proxy.socks_listen",
                "proxy.dns_listen",
            ],
        );
    }
}
