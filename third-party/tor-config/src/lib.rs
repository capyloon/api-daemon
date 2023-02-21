#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
//! `tor-config`: Tools for configuration management in Arti
//!
//! # Overview
//!
//! This crate is part of
//! [Arti](https://gitlab.torproject.org/tpo/core/arti/), a project to
//! implement [Tor](https://www.torproject.org/) in Rust.
//!
//! It provides types for handling configuration values,
//! and general machinery for configuration management.
//!
//! # Configuration in Arti
//!
//! The configuration for the `arti` command line program,
//! and other programs which embed Arti reusing the configuration machinery,
//! works as follows:
//!
//!  1. We use [`tor_config::ConfigurationSources`](ConfigurationSources)
//!     to enumerate the various places
//!     where configuration information needs to come from,
//!     and configure how they are to be read.
//!     `arti` uses [`ConfigurationSources::from_cmdline`].
//!
//!  2. [`ConfigurationSources::load`] actually *reads* all of these sources,
//!     parses them (eg, as TOML files),
//!     and returns a [`config::Config`].
//!     This is a tree-structured dynamically typed data structure,
//!     mirroring the input configuration structure, largely unvalidated,
//!     and containing everything in the input config sources.
//!
//!  3. We call one of the [`tor_config::resolve`](resolve) family.
//!     This maps the input configuration data to concrete `ConfigBuilder `s
//!     for the configuration consumers within the program.
//!     (For `arti`, that's `TorClientConfigBuilder` and `ArtiBuilder`).
//!     This mapping is done using the `Deserialize` implementations on the `Builder`s.
//!     `resolve` then calls the `build()` method on each of these parts of the configuration
//!     which applies defaults and validates the resulting configuration.
//!
//!     It is important to call `resolve` *once* for *all* the configuration consumers,
//!     so that it sees a unified view of which config settings in the input
//!     were unrecognized, and therefore may need to be reported to the user.
//!     See the example in the [`load`] module documentation.
//!
//!  4. The resulting configuration objects (eg, `TorClientConfig`, `ArtiConfig`)
//!     are provided to the code that must use them (eg, to make a `TorClient`).
//!
//! See the
//! [`tor_config::load` module-level documentation](load).
//! for an example.
//!
//! # ⚠ Stability Warning ⚠
//!
//! The design of this crate, and of the configuration system for
//! Arti, is likely to change significantly before the release of Arti
//! 1.0.0.  For more information see ticket [#285].
//!
//! [#285]: https://gitlab.torproject.org/tpo/core/arti/-/issues/285

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
#![warn(clippy::rc_buffer)]
#![deny(clippy::ref_option_ref)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![warn(clippy::trait_duplication_in_bounds)]
#![deny(clippy::unnecessary_wraps)]
#![warn(clippy::unseparated_literal_suffix)]
#![deny(clippy::unwrap_used)]
#![allow(clippy::let_unit_value)] // This can reasonably be done for explicitness
#![allow(clippy::significant_drop_in_scrutinee)] // arti/-/merge_requests/588/#note_2812945
//! <!-- @@ end lint list maintained by maint/add_warning @@ -->

pub mod cmdline;
mod err;
pub mod list_builder;
pub mod load;
mod misc;
mod mut_cfg;
mod path;
pub mod sources;

pub use cmdline::CmdLine;
pub use config as config_crate;
pub use educe;
pub use err::{ConfigBuildError, ConfigError, ReconfigureError};
pub use itertools::Itertools;
pub use load::{resolve, resolve_ignore_warnings, resolve_return_results};
pub use misc::*;
pub use mut_cfg::MutCfg;
pub use paste::paste;
pub use path::{CfgPath, CfgPathError};
pub use serde;
pub use sources::{ConfigurationSource, ConfigurationSources};

pub use tor_basic_utils::macro_first_nonempty;

/// Rules for reconfiguring a running Arti instance.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[non_exhaustive]
pub enum Reconfigure {
    /// Perform no reconfiguration unless we can guarantee that all changes will be successful.
    AllOrNothing,
    /// Try to reconfigure as much as possible; warn on fields that we cannot reconfigure.
    WarnOnFailures,
    /// Don't reconfigure anything: Only check whether we can guarantee that all changes will be successful.
    CheckAllOrNothing,
}

impl Reconfigure {
    /// Called when we see a disallowed attempt to change `field`: either give a ReconfigureError,
    /// or warn and return `Ok(())`, depending on the value of `self`.
    pub fn cannot_change<S: AsRef<str>>(self, field: S) -> Result<(), ReconfigureError> {
        match self {
            Reconfigure::AllOrNothing | Reconfigure::CheckAllOrNothing => {
                Err(ReconfigureError::CannotChange {
                    field: field.as_ref().to_owned(),
                })
            }
            Reconfigure::WarnOnFailures => {
                tracing::warn!("Cannot change {} on a running client.", field.as_ref());
                Ok(())
            }
        }
    }
}

/// Resolves an `Option<Option<T>>` (in a builder) into an `Option<T>`
///
///  * If the input is `None`, this indicates that the user did not specify a value,
///    and we therefore use `def` to obtain the default value.
///
///  * If the input is `Some(None)`, or `Some(Some(Default::default()))`,
///    the user has explicitly specified that this config item should be null/none/nothing,
///    so we return `None`.
///
///  * Otherwise the user provided an actual value, and we return `Some` of it.
///
/// See <https://gitlab.torproject.org/tpo/core/arti/-/issues/488>
///
/// For consistency with other APIs in Arti, when using this,
/// do not pass `setter(strip_option)` to derive_builder.
///
/// # ⚠ Stability Warning ⚠
///
/// We may significantly change this so that it is an method in an extension trait.
//
// This is an annoying AOI right now because you have to write things like
//     #[builder(field(build = r#"tor_config::resolve_option(&self.dns_port, || None)"#))]
//     pub(crate) dns_port: Option<u16>,
// which recapitulates the field name.  That is very much a bug hazard (indeed, in an
// early version of some of this code I perpetrated precisely that bug).
// Fixing this involves a derive_builder feature.
pub fn resolve_option<T, DF>(input: &Option<Option<T>>, def: DF) -> Option<T>
where
    T: Clone + Default + PartialEq,
    DF: FnOnce() -> Option<T>,
{
    resolve_option_general(
        input.as_ref().map(|ov| ov.as_ref()),
        |v| v == &T::default(),
        def,
    )
}

/// Resolves an `Option<Option<T>>` (in a builder) into an `Option<T>`, more generally
///
/// Like `resolve_option`, but:
///
///  * Doesn't rely on `T`' being `Default + PartialEq`
///    to determine whether it's the sentinel value;
///    instead, taking `is_explicit`.
///
///  * Takes `Option<Option<&T>>` which is more general, but less like the usual call sites.
///
///  * If the input is `None`, this indicates that the user did not specify a value,
///    and we therefore use `def` to obtain the default value.
///
///  * If the input is `Some(None)`, or `Some(Some(v)) where is_sentinel(v)`,
///    the user has explicitly specified that this config item should be null/none/nothing,
///    so we return `None`.
///
///  * Otherwise the user provided an actual value, and we return `Some` of it.
///
/// See <https://gitlab.torproject.org/tpo/core/arti/-/issues/488>
///
/// # ⚠ Stability Warning ⚠
///
/// We may significantly change this so that it is an method in an extension trait.
//
// TODO: it would be nice to have an example here, but right now I'm not sure
// what type (or config setting) we could put in an example that would be natural enough
// to add clarity.  See
//  https://gitlab.torproject.org/tpo/core/arti/-/merge_requests/685#note_2829951
pub fn resolve_option_general<T, ISF, DF>(
    input: Option<Option<&T>>,
    is_sentinel: ISF,
    def: DF,
) -> Option<T>
where
    T: Clone,
    DF: FnOnce() -> Option<T>,
    ISF: FnOnce(&T) -> bool,
{
    match input {
        None => def(),
        Some(None) => None,
        Some(Some(v)) if is_sentinel(v) => None,
        Some(Some(v)) => Some(v.clone()),
    }
}

/// Helper for resolving a config item which can be specified in multiple ways
///
/// Useable when a single configuration item can be specified
/// via multiple (alternative) input fields;
/// Each input field which is actually present
/// should be converted to the common output type,
/// and then passed to this function,
/// which will handle consistency checks and defaulting.
///
/// A common use case is deprecated field name/types.
/// In that case, the deprecated field names should be added to the appropriate
/// [`load::TopLevel::DEPRECATED_KEYS`].
///
/// `specified` should be an array (or other iterator) of `(key, Option<value>)`
/// where `key` is the field name and
/// `value` is that field from the builder,
/// converted to the common output type `V`.
///
/// # Example
///
/// ```
/// use derive_builder::Builder;
/// use serde::{Deserialize, Serialize};
/// use tor_config::{impl_standard_builder, ConfigBuildError, Listen, resolve_alternative_specs};
///
/// #[derive(Debug, Clone, Builder, Eq, PartialEq)]
/// #[builder(build_fn(error = "ConfigBuildError"))]
/// #[builder(derive(Debug, Serialize, Deserialize))]
/// #[allow(clippy::option_option)]
/// pub struct ProxyConfig {
///    /// Addresses to listen on for incoming SOCKS connections.
///    #[builder(field(build = r#"self.resolve_socks_port()?"#))]
///    pub(crate) socks_listen: Listen,
///
///    /// Port to listen on (at localhost) for incoming SOCKS
///    /// connections.
///    #[builder(setter(strip_option), field(type = "Option<Option<u16>>", build = "()"))]
///    pub(crate) socks_port: (),
/// }
/// impl_standard_builder! { ProxyConfig }
///
/// impl ProxyConfigBuilder {
///     fn resolve_socks_port(&self) -> Result<Listen, ConfigBuildError> {
///         resolve_alternative_specs(
///             [
///                 ("socks_listen", self.socks_listen.clone()),
///                 ("socks_port", self.socks_port.map(Listen::new_localhost_optional)),
///             ],
///             || Listen::new_localhost(9150),
///         )
///     }
/// }
/// ```
//
// Testing: this is tested quit exhaustively in the context of the listen/port handling, in
// crates/arti/src/cfg.rs.
pub fn resolve_alternative_specs<V, K>(
    specified: impl IntoIterator<Item = (K, Option<V>)>,
    default: impl FnOnce() -> V,
) -> Result<V, ConfigBuildError>
where
    K: Into<String>,
    V: Eq,
{
    Ok(specified
        .into_iter()
        .filter_map(|(k, v)| Some((k, v?)))
        .dedup_by(|(_, v1), (_, v2)| v1 == v2)
        .at_most_one()
        .map_err(|several| ConfigBuildError::Inconsistent {
            fields: several.into_iter().map(|(k, _v)| k.into()).collect_vec(),
            problem: "conflicting fields, specifying different values".into(),
        })?
        .map(|(_k, v)| v)
        .unwrap_or_else(default))
}

/// Defines standard impls for a struct with a `Builder`, incl `Default`
///
/// **Use this.**  Do not `#[derive(Builder, Default)]`.  That latter approach would produce
/// wrong answers if builder attributes are used to specify non-`Default` default values.
///
/// # Input syntax
///
/// ```
/// use derive_builder::Builder;
/// use serde::{Deserialize, Serialize};
/// use tor_config::impl_standard_builder;
/// use tor_config::ConfigBuildError;
///
/// #[derive(Debug, Builder, Clone, Eq, PartialEq)]
/// #[builder(derive(Serialize, Deserialize, Debug))]
/// #[builder(build_fn(error = "ConfigBuildError"))]
/// struct SomeConfigStruct { }
/// impl_standard_builder! { SomeConfigStruct }
///
/// #[derive(Debug, Builder, Clone, Eq, PartialEq)]
/// struct UnusualStruct { }
/// impl_standard_builder! { UnusualStruct: !Deserialize + !Builder }
/// ```
///
/// # Requirements
///
/// `$Config`'s builder must have default values for all the fields,
/// or this macro-generated self-test will fail.
/// This should be OK for all principal elements of our configuration.
///
/// `$ConfigBuilder` must have an appropriate `Deserialize` impl.
///
/// # Options
///
///  * `!Default` suppresses the `Default` implementation, and the corresponding tests.
///    This should be done within Arti's configuration only for sub-structures which
///    contain mandatory fields (and are themselves optional).
///
///  * `!Deserialize` suppresses the test case involving `Builder: Deserialize`.
///    This should not be done for structs which are part of Arti's configuration,
///    but can be appropriate for other types that use [`derive_builder`].
///
///  * `!Builder` suppresses the impl of the [`tor_config::load::Builder`](load::Builder) trait
///    This will be necessary if the error from the builder is not [`ConfigBuildError`].
///
/// # Generates
///
///  * `impl Default for $Config`
///  * `impl Builder for $ConfigBuilder`
///  * a self-test that the `Default` impl actually works
///  * a test that the `Builder` can be deserialized from an empty [`config::Config`],
///    and then built, and that the result is the same as the ordinary default.
//
// The implementation munches fake "trait bounds" (`: !Deserialize + !Wombat ...`) off the RHS.
// We're going to add at least one more option.
//
// When run with `!Default`, this only generates a `builder` impl and an impl of
// the `Resolvable` trait which probably won't be used anywhere.  That may seem
// like a poor tradeoff (much fiddly macro code to generate a trivial function in
// a handful of call sites).  However, this means that `impl_standard_builder!`
// can be used in more places.  That sets a good example: always use the macro.
//
// That is a good example because we want `impl_standard_builder!` to be
// used elsewhere because it generates necessary tests of properties
// which might otherwise be violated.  When adding code, people add according to the
// patterns they see.
//
// (We, sadly, don't have a good way to *ensure* use of `impl_standard_builder`.)
#[macro_export]
macro_rules! impl_standard_builder {
    // Convert the input into the "being processed format":
    {
        $Config:ty $(: $($options:tt)* )?
    } => { $crate::impl_standard_builder!{
        // ^Being processed format:
        @ ( Builder                    )
          ( default                    )
          ( try_deserialize            ) $Config    :                 $( $( $options    )* )?
        //  ~~~~~~~~~~~~~~~              ^^^^^^^    ^   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        // present iff not !Builder, !Default
        // present iff not !Default
        // present iff not !Deserialize  type      always present    options yet to be parsed
    } };
    // If !Deserialize is the next option, implement it by making $try_deserialize absent
    {
        @ ( $($Builder        :ident)? )
          ( $($default        :ident)? )
          ( $($try_deserialize:ident)? ) $Config:ty : $(+)? !Deserialize $( $options:tt )*
    } => {  $crate::impl_standard_builder!{
        @ ( $($Builder              )? )
          ( $($default              )? )
          (                            ) $Config    :                    $( $options    )*
    } };
    // If !Builder is the next option, implement it by making $Builder absent
    {
        @ ( $($Builder        :ident)? )
          ( $($default        :ident)? )
          ( $($try_deserialize:ident)? ) $Config:ty : $(+)? !Builder     $( $options:tt )*
    } => {  $crate::impl_standard_builder!{
        @ (                            )
          ( $($default              )? )
          ( $($try_deserialize      )? ) $Config    :                    $( $options    )*
    } };
    // If !Default is the next option, implement it by making $default absent
    {
        @ ( $($Builder        :ident)? )
          ( $($default        :ident)? )
          ( $($try_deserialize:ident)? ) $Config:ty : $(+)? !Default     $( $options:tt )*
    } => {  $crate::impl_standard_builder!{
        @ ( $($Builder              )? )
          (                            )
          ( $($try_deserialize      )? ) $Config    :                    $( $options    )*
    } };
    // Having parsed all options, produce output:
    {
        @ ( $($Builder        :ident)? )
          ( $($default        :ident)? )
          ( $($try_deserialize:ident)? ) $Config:ty : $(+)?
    } => { $crate::paste!{
        impl $Config {
            /// Returns a fresh, default, builder
            pub fn builder() -> [< $Config Builder >] {
                Default::default()
            }
        }

        $( // expands iff there was $default, which is always default
            impl Default for $Config {
                fn $default() -> Self {
                    // unwrap is good because one of the test cases above checks that it works!
                    [< $Config Builder >]::default().build().unwrap()
                }
            }
        )?

        $( // expands iff there was $Builder, which is always Builder
            impl $crate::load::$Builder for [< $Config Builder >] {
                type Built = $Config;
                fn build(&self) -> std::result::Result<$Config, $crate::ConfigBuildError> {
                    [< $Config Builder >]::build(self)
                }
            }
        )?

        #[test]
        #[allow(non_snake_case)]
        fn [< test_impl_Default_for_ $Config >] () {
            #[allow(unused_variables)]
            let def = None::<$Config>;
            $( // expands iff there was $default, which is always default
                let def = Some($Config::$default());
            )?

            if let Some(def) = def {
                $( // expands iff there was $try_deserialize, which is always try_deserialize
                    let empty_config = $crate::config_crate::Config::builder().build().unwrap();
                    let builder: [< $Config Builder >] = empty_config.$try_deserialize().unwrap();
                    let from_empty = builder.build().unwrap();
                    assert_eq!(def, from_empty);
                )*
            }
        }
    } };
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // why is this not the default in tests
mod test {
    use super::*;
    use crate as tor_config;
    use derive_builder::Builder;
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use tracing_test::traced_test;

    #[test]
    #[traced_test]
    fn reconfigure_helpers() {
        let how = Reconfigure::AllOrNothing;
        let err = how.cannot_change("the_laws_of_physics").unwrap_err();
        assert_eq!(
            err.to_string(),
            "Cannot change the_laws_of_physics on a running client.".to_owned()
        );

        let how = Reconfigure::WarnOnFailures;
        let ok = how.cannot_change("stuff");
        assert!(ok.is_ok());
        assert!(logs_contain("Cannot change stuff on a running client."));
    }

    #[test]
    #[rustfmt::skip] // autoformatting obscures the regular structure
    fn resolve_option_test() {
        #[derive(Debug, Clone, Builder, Eq, PartialEq)]
        #[builder(build_fn(error = "ConfigBuildError"))]
        #[builder(derive(Debug, Serialize, Deserialize, Eq, PartialEq))]
        struct TestConfig {
            #[builder(field(build = r#"tor_config::resolve_option(&self.none, || None)"#))]
            none: Option<u32>,

            #[builder(field(build = r#"tor_config::resolve_option(&self.four, || Some(4))"#))]
            four: Option<u32>,
        }

        // defaults
        {
            let builder_from_json: TestConfigBuilder = serde_json::from_value(
                json!{ { } }
            ).unwrap();

            let builder_from_methods = TestConfigBuilder::default();

            assert_eq!(builder_from_methods, builder_from_json);
            assert_eq!(builder_from_methods.build().unwrap(),
                        TestConfig { none: None, four: Some(4) });
        }

        // explicit positive values
        {
            let builder_from_json: TestConfigBuilder = serde_json::from_value(
                json!{ { "none": 123, "four": 456 } }
            ).unwrap();

            let mut builder_from_methods = TestConfigBuilder::default();
            builder_from_methods.none(Some(123));
            builder_from_methods.four(Some(456));

            assert_eq!(builder_from_methods, builder_from_json);
            assert_eq!(builder_from_methods.build().unwrap(),
                       TestConfig { none: Some(123), four: Some(456) });
        }

        // explicit "null" values
        {
            let builder_from_json: TestConfigBuilder = serde_json::from_value(
                json!{ { "none": 0, "four": 0 } }
            ).unwrap();

            let mut builder_from_methods = TestConfigBuilder::default();
            builder_from_methods.none(Some(0));
            builder_from_methods.four(Some(0));

            assert_eq!(builder_from_methods, builder_from_json);
            assert_eq!(builder_from_methods.build().unwrap(),
                       TestConfig { none: None, four: None });
        }

        // explicit None (API only, serde can't do this for Option)
        {
            let mut builder_from_methods = TestConfigBuilder::default();
            builder_from_methods.none(None);
            builder_from_methods.four(None);

            assert_eq!(builder_from_methods.build().unwrap(),
                       TestConfig { none: None, four: None });
        }
    }
}
