//! Types and functions to configure a Tor client.
//!
//! Some of these are re-exported from lower-level crates.

use derive_builder::Builder;
use derive_more::AsRef;
use fs_mistrust::{Mistrust, MistrustBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
pub use tor_chanmgr::{ChannelConfig, ChannelConfigBuilder};
pub use tor_config::convert_helper_via_multi_line_list_builder;
pub use tor_config::impl_standard_builder;
pub use tor_config::list_builder::{MultilineListBuilder, MultilineListBuilderError};
pub use tor_config::{define_list_builder_accessors, define_list_builder_helper};
pub use tor_config::{BoolOrAuto, ConfigError};
pub use tor_config::{CfgPath, CfgPathError, ConfigBuildError, ConfigurationSource, Reconfigure};

pub use tor_guardmgr::bridge::BridgeConfigBuilder;

#[cfg(feature = "bridge-client")]
#[cfg_attr(docsrs, doc(cfg(feature = "bridge-client")))]
pub use tor_guardmgr::bridge::BridgeParseError;

#[cfg(feature = "experimental-api")]
#[cfg_attr(docsrs, doc(cfg(feature = "experimental-api")))]
pub use tor_config::ItemOrBool;

use tor_guardmgr::bridge::BridgeConfig;

/// Types for configuring how Tor circuits are built.
pub mod circ {
    pub use tor_circmgr::{
        CircMgrConfig, CircuitTiming, CircuitTimingBuilder, PathConfig, PathConfigBuilder,
        PreemptiveCircuitConfig, PreemptiveCircuitConfigBuilder,
    };
}

/// Types for configuring how Tor accesses its directory information.
pub mod dir {
    pub use tor_dirmgr::{
        Authority, AuthorityBuilder, DirMgrConfig, DirTolerance, DirToleranceBuilder,
        DownloadSchedule, DownloadScheduleConfig, DownloadScheduleConfigBuilder, FallbackDir,
        FallbackDirBuilder, NetworkConfig, NetworkConfigBuilder,
    };
}

/// Types for configuring pluggable transports.
#[cfg(feature = "pt-client")]
pub mod pt {
    pub use tor_ptmgr::config::{ManagedTransportConfig, ManagedTransportConfigBuilder};
}

/// Configuration for client behavior relating to addresses.
///
/// This type is immutable once constructed. To create an object of this type,
/// use [`ClientAddrConfigBuilder`].
///
/// You can replace this configuration on a running Arti client.  Doing so will
/// affect new streams and requests, but will have no effect on existing streams
/// and requests.
#[derive(Debug, Clone, Builder, Eq, PartialEq)]
#[builder(build_fn(error = "ConfigBuildError"))]
#[builder(derive(Debug, Serialize, Deserialize))]
pub struct ClientAddrConfig {
    /// Should we allow attempts to make Tor connections to local addresses?
    ///
    /// This option is off by default, since (by default) Tor exits will
    /// always reject connections to such addresses.
    #[builder(default)]
    pub(crate) allow_local_addrs: bool,

    /// Should we allow attempts to connect to hidden services (`.onion` services)?
    ///
    /// This option is on by default.
    #[cfg(feature = "onion-service-client")]
    #[builder(default = "false")]
    pub(crate) allow_onion_addrs: bool,
}
impl_standard_builder! { ClientAddrConfig }

/// Configuration for client behavior relating to stream connection timeouts
///
/// This type is immutable once constructed. To create an object of this type,
/// use [`StreamTimeoutConfigBuilder`].
///
/// You can replace this configuration on a running Arti client.  Doing so will
/// affect new streams and requests, but will have no effect on existing streams
/// and requests—even those that are currently waiting.
#[derive(Debug, Clone, Builder, Eq, PartialEq)]
#[builder(build_fn(error = "ConfigBuildError"))]
#[builder(derive(Debug, Serialize, Deserialize))]
#[non_exhaustive]
pub struct StreamTimeoutConfig {
    /// How long should we wait before timing out a stream when connecting
    /// to a host?
    #[builder(default = "default_connect_timeout()")]
    #[builder_field_attr(serde(default, with = "humantime_serde::option"))]
    pub(crate) connect_timeout: Duration,

    /// How long should we wait before timing out when resolving a DNS record?
    #[builder(default = "default_dns_resolve_timeout()")]
    #[builder_field_attr(serde(default, with = "humantime_serde::option"))]
    pub(crate) resolve_timeout: Duration,

    /// How long should we wait before timing out when resolving a DNS
    /// PTR record?
    #[builder(default = "default_dns_resolve_ptr_timeout()")]
    #[builder_field_attr(serde(default, with = "humantime_serde::option"))]
    pub(crate) resolve_ptr_timeout: Duration,
}
impl_standard_builder! { StreamTimeoutConfig }

/// Return the default stream timeout
fn default_connect_timeout() -> Duration {
    Duration::new(10, 0)
}

/// Return the default resolve timeout
fn default_dns_resolve_timeout() -> Duration {
    Duration::new(10, 0)
}

/// Return the default PTR resolve timeout
fn default_dns_resolve_ptr_timeout() -> Duration {
    Duration::new(10, 0)
}

/// Extension trait for `MistrustBuilder` to convert the error type on
/// build.
trait BuilderExt {
    /// Type that this builder provides.
    type Built;
    /// Run this builder and convert its error type (if any)
    fn build_for_arti(&self) -> Result<Self::Built, ConfigBuildError>;
}

impl BuilderExt for MistrustBuilder {
    type Built = Mistrust;

    fn build_for_arti(&self) -> Result<Self::Built, ConfigBuildError> {
        self.clone()
            .controlled_by_env_var_if_not_set(FS_PERMISSIONS_CHECKS_DISABLE_VAR)
            .build()
            .map_err(|e| ConfigBuildError::Invalid {
                field: "permissions".to_string(),
                problem: e.to_string(),
            })
    }
}

/// Configuration for where information should be stored on disk.
///
/// By default, cache information will be stored in `${ARTI_CACHE}`, and
/// persistent state will be stored in `${ARTI_LOCAL_DATA}`.  That means that
/// _all_ programs using these defaults will share their cache and state data.
/// If that isn't what you want,  you'll need to override these directories.
///
/// On unix, the default directories will typically expand to `~/.cache/arti`
/// and `~/.local/share/arti/` respectively, depending on the user's
/// environment. Other platforms will also use suitable defaults. For more
/// information, see the documentation for [`CfgPath`].
///
/// This section is for read/write storage.
///
/// You cannot change this section on a running Arti client.
#[derive(Debug, Clone, Builder, Eq, PartialEq)]
#[builder(build_fn(error = "ConfigBuildError"))]
#[builder(derive(Debug, Serialize, Deserialize))]
#[non_exhaustive]
pub struct StorageConfig {
    /// Location on disk for cached directory information.
    #[builder(setter(into), default = "default_cache_dir()")]
    cache_dir: CfgPath,
    /// Location on disk for less-sensitive persistent state information.
    #[builder(setter(into), default = "default_state_dir()")]
    state_dir: CfgPath,
    /// Location on disk for the Arti keystore.
    //
    // TODO HSS: try to use #[serde(into / try_from)] instead (and also move the deserialization
    // code to tor-keymgr).
    #[cfg(feature = "experimental-api")] // TODO HSS: make this unconditional
    #[builder(setter(into), default = "default_keystore_dir()")]
    #[builder_field_attr(serde(
        deserialize_with = "deserialize_keystore_dir",
        default = "serde_default_keystore_dir"
    ))]
    keystore_dir: Option<CfgPath>,
    /// Filesystem state to
    #[builder(sub_builder(fn_name = "build_for_arti"))]
    #[builder_field_attr(serde(default))]
    permissions: Mistrust,
}
impl_standard_builder! { StorageConfig }

/// Deserialize the `keystore_dir` storage field.
///
/// NOTE: The first option is needed because of the builder, the second is the actual type of the field.
#[cfg(all(feature = "keymgr", feature = "experimental-api"))]
#[allow(clippy::option_option)]
fn deserialize_keystore_dir<'de, D>(deserializer: D) -> Result<Option<Option<CfgPath>>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let cfg: ItemOrBool<CfgPath> = serde::de::Deserialize::deserialize(deserializer)?;

    let cfg = match cfg {
        ItemOrBool::Item(cfg) => Some(cfg),
        ItemOrBool::Bool(true) => default_keystore_dir(),
        ItemOrBool::Bool(false) => None,
    };

    Ok(Some(cfg))
}

/// Deserialize the `keystore_dir` storage field.
///
/// NOTE: The first option is needed because of the builder, the second is the actual type of the field.
#[cfg(all(not(feature = "keymgr"), feature = "experimental-api"))]
#[allow(clippy::option_option)]
fn deserialize_keystore_dir<'de, D>(deserializer: D) -> Result<Option<Option<CfgPath>>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    use serde::de::Error as _;

    let cfg: ItemOrBool<CfgPath> = serde::de::Deserialize::deserialize(deserializer)?;

    match cfg {
        ItemOrBool::Item(_) | ItemOrBool::Bool(true) => Err(D::Error::custom(
            "keystore not available unless the `keymgr` feature is enabled".to_string(),
        )),
        ItemOrBool::Bool(false) => Ok(None),
    }
}

/// Return the default cache directory.
fn default_cache_dir() -> CfgPath {
    CfgPath::new("${ARTI_CACHE}".to_owned())
}

/// Return the default state directory.
fn default_state_dir() -> CfgPath {
    CfgPath::new("${ARTI_LOCAL_DATA}".to_owned())
}

/// Return the default keystore directory.
#[cfg(feature = "experimental-api")]
#[allow(clippy::unnecessary_wraps)] // needed because of the type expected by the builder
fn default_keystore_dir() -> Option<CfgPath> {
    #[cfg(feature = "keymgr")]
    {
        Some(CfgPath::new("${ARTI_LOCAL_DATA}/keystore".to_owned()))
    }

    #[cfg(not(feature = "keymgr"))]
    {
        None
    }
}

/// Return the default keystore directory.
#[cfg(feature = "experimental-api")]
#[allow(clippy::unnecessary_wraps, clippy::option_option)] // needed because of the type expected by the builder
fn serde_default_keystore_dir() -> Option<Option<CfgPath>> {
    Some(default_keystore_dir())
}

// TODO: the expand_* functions are very repetitive. Maybe we can generate them using a macro
// instead?
impl StorageConfig {
    /// Try to expand `state_dir` to be a path buffer.
    pub(crate) fn expand_state_dir(&self) -> Result<PathBuf, ConfigBuildError> {
        self.state_dir
            .path()
            .map_err(|e| ConfigBuildError::Invalid {
                field: "state_dir".to_owned(),
                problem: e.to_string(),
            })
    }
    /// Try to expand `cache_dir` to be a path buffer.
    pub(crate) fn expand_cache_dir(&self) -> Result<PathBuf, ConfigBuildError> {
        self.cache_dir
            .path()
            .map_err(|e| ConfigBuildError::Invalid {
                field: "cache_dir".to_owned(),
                problem: e.to_string(),
            })
    }
    /// Try to expand `keystore_dir` to be a path buffer.
    #[allow(clippy::unnecessary_wraps)] // needed because of the experimental-api branch
    pub(crate) fn expand_keystore_dir(&self) -> Result<Option<PathBuf>, ConfigBuildError> {
        #[cfg(feature = "experimental-api")]
        {
            self.keystore_dir
                .as_ref()
                .map(|dir| dir.path())
                .transpose()
                .map_err(|e| ConfigBuildError::Invalid {
                    field: "keystore_dir".to_owned(),
                    problem: e.to_string(),
                })
        }

        #[cfg(not(feature = "experimental-api"))]
        Ok(None)
    }
    /// Return the FS permissions to use for state and cache directories.
    pub(crate) fn permissions(&self) -> &Mistrust {
        &self.permissions
    }
}

/// Configuration for anti-censorship features: bridges and pluggable transports.
///
/// A "bridge" is a relay that is not listed in the regular Tor network directory;
/// clients use them to reach the network when a censor is blocking their
/// connection to all the regular Tor relays.
///
/// A "pluggable transport" is a tool that transforms and conceals a user's connection
/// to a bridge; clients use them to reach the network when a censor is blocking
/// all traffic that "looks like Tor".
///
/// A [`BridgesConfig`] configuration has the following pieces:
///    * A [`BridgeList`] of [`BridgeConfig`]s, which describes one or more bridges.
///    * An `enabled` boolean to say whether or not to use the listed bridges.
///    * A list of [`pt::ManagedTransportConfig`]s.
///
/// # Example
///
/// Here's an example of building a bridge configuration, and using it in a
/// TorClientConfig.
///
/// The bridges here are fictitious; you'll need to use real bridges
/// if you want a working configuration.
///
/// ```
/// ##[cfg(feature = "pt-client")]
/// # fn demo() -> anyhow::Result<()> {
/// use arti_client::config::{TorClientConfig, BridgeConfigBuilder, CfgPath};
/// // Requires that the pt-client feature is enabled.
/// use arti_client::config::pt::ManagedTransportConfigBuilder;
///
/// let mut builder = TorClientConfig::builder();
///
/// // Add a single bridge to the list of bridges, from a bridge line.
/// // This bridge line is made up for demonstration, and won't work.
/// const BRIDGE1_LINE : &str = "Bridge obfs4 192.0.2.55:38114 316E643333645F6D79216558614D3931657A5F5F cert=YXJlIGZyZXF1ZW50bHkgZnVsbCBvZiBsaXR0bGUgbWVzc2FnZXMgeW91IGNhbiBmaW5kLg iat-mode=0";
/// let bridge_1: BridgeConfigBuilder = BRIDGE1_LINE.parse()?;
/// // This is where we pass `BRIDGE1_LINE` into the BridgeConfigBuilder.
/// builder.bridges().bridges().push(bridge_1);
///
/// // Add a second bridge, built by hand.  This way is harder.
/// // This bridge is made up for demonstration, and won't work.
/// let mut bridge2_builder = BridgeConfigBuilder::default();
/// bridge2_builder
///     .transport("obfs4")
///     .push_setting("iat-mode", "1")
///     .push_setting(
///         "cert",
///         "YnV0IHNvbWV0aW1lcyB0aGV5IGFyZSByYW5kb20u8x9aQG/0cIIcx0ItBcTqiSXotQne+Q"
///     );
/// bridge2_builder.set_addrs(vec!["198.51.100.25:443".parse()?]);
/// bridge2_builder.set_ids(vec!["7DD62766BF2052432051D7B7E08A22F7E34A4543".parse()?]);
/// // Now insert the second bridge into our config builder.
/// builder.bridges().bridges().push(bridge2_builder);
///
/// // Now configure an obfs4 transport. (Requires the "pt-client" feature)
/// let mut transport = ManagedTransportConfigBuilder::default();
/// transport
///     .protocols(vec!["obfs4".parse()?])
///     // Specify either the name or the absolute path of pluggable transport client binary, this
///     // may differ from system to system.
///     .path(CfgPath::new("/usr/bin/obfs4proxy".into()))
///     .run_on_startup(true);
/// builder.bridges().transports().push(transport);
///
/// let config = builder.build()?;
/// // Now you can pass `config` to TorClient::create!
/// # Ok(())}
/// ```
/// You can also find an example based on snowflake in arti-client example folder.
//
// We leave this as an empty struct even when bridge support is disabled,
// as otherwise the default config file would generate an unknown section warning.
#[derive(Debug, Clone, Builder, Eq, PartialEq)]
#[builder(build_fn(validate = "validate_bridges_config", error = "ConfigBuildError"))]
#[builder(derive(Debug, Serialize, Deserialize))]
#[non_exhaustive]
#[builder_struct_attr(non_exhaustive)] // This struct can be empty.
pub struct BridgesConfig {
    /// Should we use configured bridges?
    ///
    /// The default (`Auto`) is to use bridges if they are configured.
    /// `false` means to not use even configured bridges.
    /// `true` means to insist on the use of bridges;
    /// if none are configured, that's then an error.
    #[builder(default)]
    pub(crate) enabled: BoolOrAuto,

    /// Configured list of bridges (possibly via pluggable transports)
    #[builder(sub_builder, setter(custom))]
    #[builder_field_attr(serde(default))]
    bridges: BridgeList,

    /// Configured list of pluggable transports.
    #[builder(sub_builder, setter(custom))]
    #[builder_field_attr(serde(default))]
    #[cfg(feature = "pt-client")]
    pub(crate) transports: TransportConfigList,
}

/// A list of configured transport binaries (type alias for macrology).
#[cfg(feature = "pt-client")]
type TransportConfigList = Vec<pt::ManagedTransportConfig>;

#[cfg(feature = "pt-client")]
define_list_builder_helper! {
    pub struct TransportConfigListBuilder {
        transports: [pt::ManagedTransportConfigBuilder],
    }
    built: TransportConfigList = transports;
    default = vec![];
}

#[cfg(feature = "pt-client")]
define_list_builder_accessors! {
    struct BridgesConfigBuilder {
        pub transports: [pt::ManagedTransportConfigBuilder],
    }
}

impl_standard_builder! { BridgesConfig }

/// Check that the bridge configuration is right
#[allow(clippy::unnecessary_wraps)]
fn validate_bridges_config(bridges: &BridgesConfigBuilder) -> Result<(), ConfigBuildError> {
    let _ = bridges; // suppresses unused variable for just that argument

    #[cfg(feature = "bridge-client")]
    use BoolOrAuto as BoA;

    // Ideally we would run this post-build, rather than pre-build;
    // doing it here means we have to recapitulate the defaulting.
    // Happily the defaulting is obvious, cheap, and not going to change.
    //
    // Alternatively we could have derive_builder provide `build_unvalidated`,
    // but that involves re-setting the build fn name for every field.
    #[cfg(feature = "bridge-client")]
    match (
        bridges.enabled.unwrap_or_default(),
        bridges.bridges.bridges.as_deref().unwrap_or_default(),
    ) {
        (BoA::Auto, _) | (BoA::Explicit(false), _) | (BoA::Explicit(true), [_, ..]) => {}
        (BoA::Explicit(true), []) => {
            return Err(ConfigBuildError::Inconsistent {
                fields: ["enabled", "bridges"].map(Into::into).into_iter().collect(),
                problem: "bridges enabled=true, but no bridges defined".into(),
            })
        }
    }

    Ok(())
}

impl BridgesConfig {
    /// Should the bridges be used?
    fn bridges_enabled(&self) -> bool {
        #[cfg(feature = "bridge-client")]
        {
            self.enabled.as_bool().unwrap_or(!self.bridges.is_empty())
        }

        #[cfg(not(feature = "bridge-client"))]
        {
            false
        }
    }
}

/// List of configured bridges, as found in the built configuration
//
// This type alias arranges that we can put `BridgeList` in `BridgesConfig`
// and have derive_builder put a `BridgeListBuilder` in `BridgesConfigBuilder`.
pub type BridgeList = Vec<BridgeConfig>;

define_list_builder_helper! {
    struct BridgeListBuilder {
        bridges: [BridgeConfigBuilder],
    }
    built: BridgeList = bridges;
    default = vec![];
    #[serde(try_from="MultilineListBuilder<BridgeConfigBuilder>")]
    #[serde(into="MultilineListBuilder<BridgeConfigBuilder>")]
}

convert_helper_via_multi_line_list_builder! {
    struct BridgeListBuilder {
        bridges: [BridgeConfigBuilder],
    }
}

#[cfg(feature = "bridge-client")]
define_list_builder_accessors! {
    struct BridgesConfigBuilder {
        pub bridges: [BridgeConfigBuilder],
    }
}

/// A configuration used to bootstrap a [`TorClient`](crate::TorClient).
///
/// In order to connect to the Tor network, Arti needs to know a few
/// well-known directory caches on the network, and the public keys of the
/// network's directory authorities.  It also needs a place on disk to
/// store persistent state and cached directory information. (See [`StorageConfig`]
/// for default directories.)
///
/// Most users will create a TorClientConfig by running
/// [`TorClientConfig::default`].
///
/// If you need to override the locations where Arti stores its
/// information, you can make a TorClientConfig with
/// [`TorClientConfigBuilder::from_directories`].
///
/// Finally, you can get fine-grained control over the members of a a
/// TorClientConfig using [`TorClientConfigBuilder`].
#[derive(Clone, Builder, Debug, Eq, PartialEq, AsRef)]
#[builder(build_fn(error = "ConfigBuildError"))]
#[builder(derive(Serialize, Deserialize, Debug))]
#[non_exhaustive]
pub struct TorClientConfig {
    /// Information about the Tor network we want to connect to.
    #[builder(sub_builder)]
    #[builder_field_attr(serde(default))]
    tor_network: dir::NetworkConfig,

    /// Directories for storing information on disk
    #[builder(sub_builder)]
    #[builder_field_attr(serde(default))]
    pub(crate) storage: StorageConfig,

    /// Information about when and how often to download directory information
    #[builder(sub_builder)]
    #[builder_field_attr(serde(default))]
    download_schedule: dir::DownloadScheduleConfig,

    /// Information about how premature or expired our directories are allowed
    /// to be.
    ///
    /// These options help us tolerate clock skew, and help survive the case
    /// where the directory authorities are unable to reach consensus for a
    /// while.
    #[builder(sub_builder)]
    #[builder_field_attr(serde(default))]
    directory_tolerance: dir::DirTolerance,

    /// Facility to override network parameters from the values set in the
    /// consensus.
    #[builder(
        sub_builder,
        field(
            type = "HashMap<String, i32>",
            build = "convert_override_net_params(&self.override_net_params)"
        )
    )]
    #[builder_field_attr(serde(default))]
    pub(crate) override_net_params: tor_netdoc::doc::netstatus::NetParams<i32>,

    /// Information about bridges, pluggable transports, and so on
    #[builder(sub_builder)]
    #[builder_field_attr(serde(default))]
    pub(crate) bridges: BridgesConfig,

    /// Information about how to build paths through the network.
    #[builder(sub_builder)]
    #[builder_field_attr(serde(default))]
    pub(crate) channel: ChannelConfig,

    /// Information about how to build paths through the network.
    #[as_ref]
    #[builder(sub_builder)]
    #[builder_field_attr(serde(default))]
    path_rules: circ::PathConfig,

    /// Information about preemptive circuits.
    #[as_ref]
    #[builder(sub_builder)]
    #[builder_field_attr(serde(default))]
    preemptive_circuits: circ::PreemptiveCircuitConfig,

    /// Information about how to retry and expire circuits and request for circuits.
    #[as_ref]
    #[builder(sub_builder)]
    #[builder_field_attr(serde(default))]
    circuit_timing: circ::CircuitTiming,

    /// Rules about which addresses the client is willing to connect to.
    #[builder(sub_builder)]
    #[builder_field_attr(serde(default))]
    pub(crate) address_filter: ClientAddrConfig,

    /// Information about timing out client requests.
    #[builder(sub_builder)]
    #[builder_field_attr(serde(default))]
    pub(crate) stream_timeouts: StreamTimeoutConfig,
}
impl_standard_builder! { TorClientConfig }

impl tor_config::load::TopLevel for TorClientConfig {
    type Builder = TorClientConfigBuilder;
}

/// Helper to convert convert_override_net_params
fn convert_override_net_params(
    builder: &HashMap<String, i32>,
) -> tor_netdoc::doc::netstatus::NetParams<i32> {
    let mut override_net_params = tor_netdoc::doc::netstatus::NetParams::new();
    for (k, v) in builder {
        override_net_params.set(k.clone(), *v);
    }
    override_net_params
}

impl tor_circmgr::CircMgrConfig for TorClientConfig {}
#[cfg(feature = "onion-service-client")]
impl tor_hsclient::HsClientConnectorConfig for TorClientConfig {}

impl AsRef<tor_guardmgr::fallback::FallbackList> for TorClientConfig {
    fn as_ref(&self) -> &tor_guardmgr::fallback::FallbackList {
        self.tor_network.fallback_caches()
    }
}
impl AsRef<[BridgeConfig]> for TorClientConfig {
    fn as_ref(&self) -> &[BridgeConfig] {
        #[cfg(feature = "bridge-client")]
        {
            &self.bridges.bridges
        }

        #[cfg(not(feature = "bridge-client"))]
        {
            &[]
        }
    }
}
impl tor_guardmgr::GuardMgrConfig for TorClientConfig {
    fn bridges_enabled(&self) -> bool {
        self.bridges.bridges_enabled()
    }
}

impl TorClientConfig {
    /// Try to create a DirMgrConfig corresponding to this object.
    #[rustfmt::skip]
    pub(crate) fn dir_mgr_config(&self) -> Result<dir::DirMgrConfig, ConfigBuildError> {
        Ok(dir::DirMgrConfig {
            network:             self.tor_network        .clone(),
            schedule:            self.download_schedule  .clone(),
            tolerance:           self.directory_tolerance.clone(),
            cache_path:          self.storage.expand_cache_dir()?,
            cache_trust:         self.storage.permissions.clone(),
            override_net_params: self.override_net_params.clone(),
            extensions:          Default::default(),
        })
    }

    /// Return a reference to the [`fs_mistrust::Mistrust`] object that we'll
    /// use to check permissions on files and directories by default.
    ///
    /// # Usage notes
    ///
    /// In the future, specific files or directories may have stricter or looser
    /// permissions checks applied to them than this default.  Callers shouldn't
    /// use this [`Mistrust`] to predict what Arti will accept for a specific
    /// file or directory.  Rather, you should use this if you have some file or
    /// directory of your own on which you'd like to enforce the same rules as
    /// Arti uses.
    //
    // NOTE: The presence of this accessor is _NOT_ in any form a commitment to
    // expose every field from the configuration as an accessor.  We explicitly
    // reject that slippery slope argument.
    pub fn fs_mistrust(&self) -> &Mistrust {
        self.storage.permissions()
    }
}

impl TorClientConfigBuilder {
    /// Returns a `TorClientConfigBuilder` using the specified state and cache directories.
    ///
    /// All other configuration options are set to their defaults.
    pub fn from_directories<P, Q>(state_dir: P, cache_dir: Q) -> Self
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        let mut builder = Self::default();
        builder
            .storage()
            .cache_dir(CfgPath::new_literal(cache_dir.as_ref()))
            .state_dir(CfgPath::new_literal(state_dir.as_ref()));
        builder
    }
}

/// Return the filenames for the default user configuration files
pub fn default_config_files() -> Result<Vec<ConfigurationSource>, CfgPathError> {
    ["${ARTI_CONFIG}/arti.toml", "${ARTI_CONFIG}/arti.d/"]
        .into_iter()
        .map(|f| {
            let path = CfgPath::new(f.into()).path()?;
            Ok(ConfigurationSource::from_path(path))
        })
        .collect()
}

/// The environment variable we look at when deciding whether to disable FS permissions checking.
pub const FS_PERMISSIONS_CHECKS_DISABLE_VAR: &str = "ARTI_FS_DISABLE_PERMISSION_CHECKS";

/// Return true if the environment has been set up to disable FS permissions
/// checking.
///
/// This function is exposed so that other tools can use the same checking rules
/// as `arti-client`.  For more information, see
/// [`TorClientBuilder`](crate::TorClientBuilder).
#[deprecated(since = "0.5.0")]
pub fn fs_permissions_checks_disabled_via_env() -> bool {
    std::env::var_os(FS_PERMISSIONS_CHECKS_DISABLE_VAR).is_some()
}

#[cfg(test)]
mod test {
    // @@ begin test lint list maintained by maint/add_warning @@
    #![allow(clippy::bool_assert_comparison)]
    #![allow(clippy::clone_on_copy)]
    #![allow(clippy::dbg_macro)]
    #![allow(clippy::print_stderr)]
    #![allow(clippy::print_stdout)]
    #![allow(clippy::single_char_pattern)]
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::unchecked_duration_subtraction)]
    //! <!-- @@ end test lint list maintained by maint/add_warning @@ -->
    use super::*;

    #[test]
    fn defaults() {
        let dflt = TorClientConfig::default();
        let b2 = TorClientConfigBuilder::default();
        let dflt2 = b2.build().unwrap();
        assert_eq!(&dflt, &dflt2);
    }

    #[test]
    fn builder() {
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

        let mut bld = TorClientConfig::builder();
        bld.tor_network().set_authorities(vec![auth]);
        bld.tor_network().set_fallback_caches(vec![fallback]);
        bld.storage()
            .cache_dir(CfgPath::new("/var/tmp/foo".to_owned()))
            .state_dir(CfgPath::new("/var/tmp/bar".to_owned()));
        bld.download_schedule().retry_certs().attempts(10);
        bld.download_schedule().retry_certs().initial_delay(sec);
        bld.download_schedule().retry_certs().parallelism(3);
        bld.download_schedule().retry_microdescs().attempts(30);
        bld.download_schedule()
            .retry_microdescs()
            .initial_delay(10 * sec);
        bld.download_schedule().retry_microdescs().parallelism(9);
        bld.override_net_params()
            .insert("wombats-per-quokka".to_owned(), 7);
        bld.path_rules()
            .ipv4_subnet_family_prefix(20)
            .ipv6_subnet_family_prefix(48);
        bld.circuit_timing()
            .max_dirtiness(90 * sec)
            .request_timeout(10 * sec)
            .request_max_retries(22)
            .request_loyalty(3600 * sec);
        bld.address_filter().allow_local_addrs(true);

        let val = bld.build().unwrap();

        assert_ne!(val, TorClientConfig::default());
    }

    #[test]
    fn check_default() {
        // We don't want to second-guess the directories crate too much
        // here, so we'll just make sure it does _something_ plausible.

        let dflt = default_config_files().unwrap();
        assert!(dflt[0].as_path().ends_with("arti.toml"));
        assert!(dflt[1].as_path().ends_with("arti.d"));
        assert_eq!(dflt.len(), 2);
    }
}
