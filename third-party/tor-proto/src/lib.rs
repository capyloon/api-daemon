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

pub mod channel;
pub mod circuit;
mod crypto;
pub mod stream;
mod util;

pub use util::err::{Error, ResolveError};
pub use util::skew::ClockSkew;

pub use channel::params::ChannelPaddingInstructions;

/// A Result type for this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Timestamp object that we update whenever we get incoming traffic.
///
/// Used to implement [`time_since_last_incoming_traffic`]
static LAST_INCOMING_TRAFFIC: util::ts::OptTimestamp = util::ts::OptTimestamp::new();

/// Called whenever we receive incoming traffic.
///
/// Used to implement [`time_since_last_incoming_traffic`]
#[inline]
pub(crate) fn note_incoming_traffic() {
    LAST_INCOMING_TRAFFIC.update();
}

/// Return the amount of time since we last received "incoming traffic".
///
/// This is a global counter, and is subject to interference from
/// other users of the `tor_proto`.  Its only permissible use is for
/// checking how recently we have been definitely able to receive
/// incoming traffic.
///
/// When enabled, this timestamp is updated whenever we receive a valid
/// cell, and whenever we complete a channel handshake.
///
/// Returns `None` if we never received "incoming traffic".
pub fn time_since_last_incoming_traffic() -> Option<std::time::Duration> {
    LAST_INCOMING_TRAFFIC.time_since_update().map(Into::into)
}
