#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
//! Represents a clients'-eye view of the Tor network.
//!
//! # Overview
//!
//! The `tor-netdir` crate wraps objects from tor-netdoc, and combines
//! them to provide a unified view of the relays on the network.
//! It is responsible for representing a client's knowledge of the
//! network's state and who is on it.
//!
//! This crate is part of
//! [Arti](https://gitlab.torproject.org/tpo/core/arti/), a project to
//! implement [Tor](https://www.torproject.org/) in Rust.  Its purpose
//! is to expose an abstract view of a Tor network and the relays in
//! it, so that higher-level crates don't need to know about the
//! particular documents that describe the network and its properties.
//!
//! There are two intended users for this crate.  First, producers
//! like [`tor-dirmgr`] create [`NetDir`] objects fill them with
//! information from the Tor network directory.  Later, consumers
//! like [`tor-circmgr`] use [`NetDir`]s to select relays for random
//! paths through the Tor network.
//!
//! # Limitations
//!
//! Only modern consensus methods and microdescriptor consensuses are
//! supported.

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

mod err;
pub mod params;
mod weight;

#[cfg(any(test, feature = "testing"))]
pub mod testnet;

use static_assertions::const_assert;
use tor_linkspec::{ChanTarget, HasAddrs, HasRelayIds, RelayIdRef, RelayIdType};
use tor_llcrypto as ll;
use tor_llcrypto::pk::{ed25519::Ed25519Identity, rsa::RsaIdentity};
use tor_netdoc::doc::microdesc::{MdDigest, Microdesc};
use tor_netdoc::doc::netstatus::{self, MdConsensus, RouterStatus};
use tor_netdoc::types::policy::PortPolicy;

use futures::stream::BoxStream;
use serde::Deserialize;
use std::collections::HashMap;
use std::net::IpAddr;
use std::ops::Deref;
use std::sync::Arc;
use tracing::warn;

pub use err::Error;
pub use weight::WeightRole;
/// A Result using the Error type from the tor-netdir crate
pub type Result<T> = std::result::Result<T, Error>;

use params::NetParameters;

/// Configuration for determining when two relays have addresses "too close" in
/// the network.
///
/// Used by [`Relay::in_same_subnet()`].
#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(deny_unknown_fields)]
pub struct SubnetConfig {
    /// Consider IPv4 nodes in the same /x to be the same family.
    ///
    /// If this value is 0, all nodes with IPv4 addresses will be in the
    /// same family.  If this value is above 32, then no nodes will be
    /// placed im the same family based on their IPv4 addresses.
    subnets_family_v4: u8,
    /// Consider IPv6 nodes in the same /x to be the same family.
    ///
    /// If this value is 0, all nodes with IPv6 addresses will be in the
    /// same family.  If this value is above 128, then no nodes will be
    /// placed im the same family based on their IPv6 addresses.
    subnets_family_v6: u8,
}

impl Default for SubnetConfig {
    fn default() -> Self {
        Self::new(16, 32)
    }
}

impl SubnetConfig {
    /// Construct a new SubnetConfig from a pair of bit prefix lengths.
    ///
    /// The values are clamped to the appropriate ranges if they are
    /// out-of-bounds.
    pub fn new(subnets_family_v4: u8, subnets_family_v6: u8) -> Self {
        Self {
            subnets_family_v4,
            subnets_family_v6,
        }
    }

    /// Are two addresses in the same subnet according to this configuration
    fn addrs_in_same_subnet(&self, a: &IpAddr, b: &IpAddr) -> bool {
        match (a, b) {
            (IpAddr::V4(a), IpAddr::V4(b)) => {
                let bits = self.subnets_family_v4;
                if bits > 32 {
                    return false;
                }
                let a = u32::from_be_bytes(a.octets());
                let b = u32::from_be_bytes(b.octets());
                (a >> (32 - bits)) == (b >> (32 - bits))
            }
            (IpAddr::V6(a), IpAddr::V6(b)) => {
                let bits = self.subnets_family_v6;
                if bits > 128 {
                    return false;
                }
                let a = u128::from_be_bytes(a.octets());
                let b = u128::from_be_bytes(b.octets());
                (a >> (128 - bits)) == (b >> (128 - bits))
            }
            _ => false,
        }
    }
}

/// An opaque type representing the weight with which a relay or set of
/// relays will be selected for a given role.
///
/// Most users should ignore this type, and just use pick_relay instead.
#[derive(
    Copy,
    Clone,
    Debug,
    derive_more::Add,
    derive_more::Sum,
    derive_more::AddAssign,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
)]
pub struct RelayWeight(u64);

impl RelayWeight {
    /// Try to divide this weight by `rhs`.
    ///
    /// Return a ratio on success, or None on division-by-zero.
    pub fn checked_div(&self, rhs: RelayWeight) -> Option<f64> {
        if rhs.0 == 0 {
            None
        } else {
            Some((self.0 as f64) / (rhs.0 as f64))
        }
    }

    /// Compute a ratio `frac` of this weight.
    ///
    /// Return None if frac is less than zero, since negative weights
    /// are impossible.
    pub fn ratio(&self, frac: f64) -> Option<RelayWeight> {
        let product = (self.0 as f64) * frac;
        if product >= 0.0 && product.is_finite() {
            Some(RelayWeight(product as u64))
        } else {
            None
        }
    }
}

impl From<u64> for RelayWeight {
    fn from(val: u64) -> Self {
        RelayWeight(val)
    }
}

/// A view of the Tor directory, suitable for use in building circuits.
///
/// Abstractly, a [`NetDir`] is a set of usable public [`Relay`]s, each of which
/// has its own properties, identity, and correct weighted probability for use
/// under different circumstances.
///
/// A [`NetDir`] is constructed by making a [`PartialNetDir`] from a consensus
/// document, and then adding enough microdescriptors to that `PartialNetDir` so
/// that it can be used to build paths. (Thus, if you have a NetDir, it is
/// definitely adequate to build paths.)
///
/// # Limitations
///
/// The current NetDir implementation assumes fairly strongly that every relay
/// has an Ed25519 identity and an RSA identity, that the consensus is indexed
/// by RSA identities, and that the Ed25519 identities are stored in
/// microdescriptors.
///
/// If these assumptions someday change, then we'll have to revise the
/// implementation.
#[derive(Debug, Clone)]
pub struct NetDir {
    /// A microdescriptor consensus that lists the members of the network,
    /// and maps each one to a 'microdescriptor' that has more information
    /// about it
    consensus: Arc<MdConsensus>,
    /// A map from keys to integer values, distributed in the consensus,
    /// and clamped to certain defaults.
    params: NetParameters,
    /// Map from  routerstatus index (the position of a routerstatus within the
    /// consensus), to that routerstatus's microdescriptor (if we have one.)
    mds: Vec<Option<Arc<Microdesc>>>,
    /// Map from SHA256 of _missing_ microdescriptors to the position of their
    /// corresponding routerstatus indices within `consensus`.
    rs_idx_by_missing: HashMap<MdDigest, usize>,
    /// Map from ed25519 identity to index of the routerstatus within
    /// `self.consensus.relays()`.
    ///
    /// Note that we don't know the ed25519 identity of a relay until
    /// we get the microdescriptor for it, so this won't be filled in
    /// until we get the microdescriptors.
    ///
    /// # Implementation note
    ///
    /// For this field, and for `rs_idx_by_rsa`, and for
    /// `MdEntry::*::rsa_idx`, it might be cool to have references instead.
    /// But that would make this into a self-referential structure,
    /// which isn't possible in safe rust.
    rs_idx_by_ed: HashMap<Ed25519Identity, usize>,
    /// Map from RSA identity to index of the routerstatus within
    /// `self.consensus.relays()`.
    ///
    /// This is constructed at the same time as the NetDir object, so it
    /// can be immutable.
    rs_idx_by_rsa: Arc<HashMap<RsaIdentity, usize>>,

    /// Weight values to apply to a given relay when deciding how frequently
    /// to choose it for a given role.
    weights: weight::WeightSet,
}

/// An event that a [`NetDirProvider`] can broadcast to indicate that a change in
/// the status of its directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DirEvent {
    /// A new consensus has been received, and has enough information to be
    /// used.
    ///
    /// This event is also broadcast when a new set of consensus parameters is
    /// available, even if that set of parameters comes from a configuration
    /// change rather than from the latest consensus.
    NewConsensus,

    /// New descriptors have been received for the current consensus.
    ///
    /// (This event is _not_ broadcast when receiving new descriptors for a
    /// consensus which is not yet ready to replace the current consensus.)
    NewDescriptors,
}

/// How "timely" must a network directory be?
///
/// This enum is used as an argument when requesting a [`NetDir`] object from
/// [`NetDirProvider`] and other APIs, to specify how recent the information
/// must be in order to be useful.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[allow(clippy::exhaustive_enums)]
pub enum Timeliness {
    /// The network directory must be strictly timely.
    ///
    /// That is, it must be based on a consensus that valid right now, with no
    /// tolerance for skew or consensus problems.
    ///
    /// Avoid using this option if you could use [`Timeliness::Timely`] instead.
    Strict,
    /// The network directory must be roughly timely.
    ///
    /// This is, it must be be based on a consensus that is not _too_ far in the
    /// future, and not _too_ far in the past.
    ///
    /// (The tolerances for "too far" will depend on configuration.)
    ///
    /// This is almost always the option that you want to use.
    Timely,
    /// Any network directory is permissible, regardless of how untimely.
    ///
    /// Avoid using this option if you could use [`Timeliness::Timely`] instead.
    Unchecked,
}

/// An object that can provide [`NetDir`]s, as well as inform consumers when
/// they might have changed.
pub trait NetDirProvider: UpcastArcNetDirProvider + Send + Sync {
    /// Return a network directory that's live according to the provided
    /// `timeliness`.
    fn netdir(&self, timeliness: Timeliness) -> Result<Arc<NetDir>>;

    /// Return a reasonable netdir for general usage.
    ///
    /// This is an alias for
    /// [`NetDirProvider::netdir`]`(`[`Timeliness::Timely`]`)`.
    fn timely_netdir(&self) -> Result<Arc<NetDir>> {
        self.netdir(Timeliness::Timely)
    }

    /// Return a new asynchronous stream that will receive notification
    /// whenever the consensus has changed.
    ///
    /// Multiple events may be batched up into a single item: each time
    /// this stream yields an event, all you can assume is that the event has
    /// occurred at least once.
    fn events(&self) -> BoxStream<'static, DirEvent>;

    /// Return the latest network parameters.
    ///
    /// If we have no directory, return a reasonable set of defaults.
    fn params(&self) -> Arc<dyn AsRef<NetParameters>>;
}

impl<T> NetDirProvider for Arc<T>
where
    T: NetDirProvider,
{
    fn netdir(&self, timeliness: Timeliness) -> Result<Arc<NetDir>> {
        self.deref().netdir(timeliness)
    }

    fn timely_netdir(&self) -> Result<Arc<NetDir>> {
        self.deref().timely_netdir()
    }

    fn events(&self) -> BoxStream<'static, DirEvent> {
        self.deref().events()
    }

    fn params(&self) -> Arc<dyn AsRef<NetParameters>> {
        self.deref().params()
    }
}

/// Helper trait: allows any `Arc<X>` to be upcast to a `Arc<dyn
/// NetDirProvider>` if X is an implementation or supertrait of NetDirProvider.
///
/// This trait exists to work around a limitation in rust: when trait upcasting
/// coercion is stable, this will be unnecessary.
///
/// The Rust tracking issue is <https://github.com/rust-lang/rust/issues/65991>.
pub trait UpcastArcNetDirProvider {
    /// Return a view of this object as an `Arc<dyn NetDirProvider>`
    fn upcast_arc<'a>(self: Arc<Self>) -> Arc<dyn NetDirProvider + 'a>
    where
        Self: 'a;
}

impl<T> UpcastArcNetDirProvider for T
where
    T: NetDirProvider + Sized,
{
    fn upcast_arc<'a>(self: Arc<Self>) -> Arc<dyn NetDirProvider + 'a>
    where
        Self: 'a,
    {
        self
    }
}

impl AsRef<NetParameters> for NetDir {
    fn as_ref(&self) -> &NetParameters {
        self.params()
    }
}

/// A partially build NetDir -- it can't be unwrapped until it has
/// enough information to build safe paths.
#[derive(Debug, Clone)]
pub struct PartialNetDir {
    /// The netdir that's under construction.
    netdir: NetDir,
}

/// A view of a relay on the Tor network, suitable for building circuits.
// TODO: This should probably be a more specific struct, with a trait
// that implements it.
#[derive(Clone)]
pub struct Relay<'a> {
    /// A router descriptor for this relay.
    rs: &'a netstatus::MdConsensusRouterStatus,
    /// A microdescriptor for this relay.
    md: &'a Microdesc,
}

/// A relay that we haven't checked for validity or usability in
/// routing.
#[derive(Debug)]
pub struct UncheckedRelay<'a> {
    /// A router descriptor for this relay.
    rs: &'a netstatus::MdConsensusRouterStatus,
    /// A microdescriptor for this relay, if there is one.
    md: Option<&'a Microdesc>,
}

/// A partial or full network directory that we can download
/// microdescriptors for.
pub trait MdReceiver {
    /// Return an iterator over the digests for all of the microdescriptors
    /// that this netdir is missing.
    fn missing_microdescs(&self) -> Box<dyn Iterator<Item = &MdDigest> + '_>;
    /// Add a microdescriptor to this netdir, if it was wanted.
    ///
    /// Return true if it was indeed wanted.
    fn add_microdesc(&mut self, md: Microdesc) -> bool;
    /// Return the number of missing microdescriptors.
    fn n_missing(&self) -> usize;
}

impl PartialNetDir {
    /// Create a new PartialNetDir with a given consensus, and no
    /// microdescriptors loaded.
    ///
    /// If `replacement_params` is provided, override network parameters from
    /// the consensus with those from `replacement_params`.
    pub fn new(
        consensus: MdConsensus,
        replacement_params: Option<&netstatus::NetParams<i32>>,
    ) -> Self {
        let mut params = NetParameters::default();

        // (We ignore unrecognized options here, since they come from
        // the consensus, and we don't expect to recognize everything
        // there.)
        let _ = params.saturating_update(consensus.params().iter());

        // Now see if the user has any parameters to override.
        // (We have to do this now, or else changes won't be reflected in our
        // weights.)
        if let Some(replacement) = replacement_params {
            for u in params.saturating_update(replacement.iter()) {
                warn!("Unrecognized option: override_net_params.{}", u);
            }
        }

        // Compute the weights we'll want to use for these relays.
        let weights = weight::WeightSet::from_consensus(&consensus, &params);

        let n_relays = consensus.relays().len();

        let rs_idx_by_missing = consensus
            .relays()
            .iter()
            .enumerate()
            .map(|(rs_idx, rs)| (*rs.md_digest(), rs_idx))
            .collect();

        let rs_idx_by_rsa = consensus
            .relays()
            .iter()
            .enumerate()
            .map(|(rs_idx, rs)| (*rs.rsa_identity(), rs_idx))
            .collect();

        let netdir = NetDir {
            consensus: Arc::new(consensus),
            params,
            mds: vec![None; n_relays],
            rs_idx_by_missing,
            rs_idx_by_rsa: Arc::new(rs_idx_by_rsa),
            rs_idx_by_ed: HashMap::with_capacity(n_relays),
            weights,
        };

        PartialNetDir { netdir }
    }

    /// Return the declared lifetime of this PartialNetDir.
    pub fn lifetime(&self) -> &netstatus::Lifetime {
        self.netdir.lifetime()
    }

    /// Fill in as many missing microdescriptors as possible in this
    /// netdir, using the microdescriptors from the previous netdir.
    pub fn fill_from_previous_netdir<'a>(&mut self, prev: &'a NetDir) -> Vec<&'a MdDigest> {
        let mut loaded = Vec::new();
        for md in prev.mds.iter().flatten() {
            if self.netdir.add_arc_microdesc(md.clone()) {
                loaded.push(md.digest());
            }
        }
        loaded
    }

    /// Return true if this are enough information in this directory
    /// to build multihop paths.
    pub fn have_enough_paths(&self) -> bool {
        self.netdir.have_enough_paths()
    }
    /// If this directory has enough information to build multihop
    /// circuits, return it.
    pub fn unwrap_if_sufficient(self) -> std::result::Result<NetDir, PartialNetDir> {
        if self.netdir.have_enough_paths() {
            Ok(self.netdir)
        } else {
            Err(self)
        }
    }
}

impl MdReceiver for PartialNetDir {
    fn missing_microdescs(&self) -> Box<dyn Iterator<Item = &MdDigest> + '_> {
        self.netdir.missing_microdescs()
    }
    fn add_microdesc(&mut self, md: Microdesc) -> bool {
        self.netdir.add_microdesc(md)
    }
    fn n_missing(&self) -> usize {
        self.netdir.n_missing()
    }
}

impl NetDir {
    /// Return the declared lifetime of this NetDir.
    pub fn lifetime(&self) -> &netstatus::Lifetime {
        self.consensus.lifetime()
    }

    /// Add `md` to this NetDir.
    ///
    /// Return true if we wanted it, and false otherwise.
    #[allow(clippy::missing_panics_doc)] // Can't panic on valid object.
    fn add_arc_microdesc(&mut self, md: Arc<Microdesc>) -> bool {
        if let Some(rs_idx) = self.rs_idx_by_missing.remove(md.digest()) {
            assert_eq!(self.consensus.relays()[rs_idx].md_digest(), md.digest());

            // There should never be two approved MDs in the same
            // consensus listing the same ID... but if there is,
            // we'll let the most recent one win.
            self.rs_idx_by_ed.insert(*md.ed25519_id(), rs_idx);

            // Happy path: we did indeed want this one.
            self.mds[rs_idx] = Some(md);

            // Save some space in the missing-descriptor list.
            if self.rs_idx_by_missing.len() < self.rs_idx_by_missing.capacity() / 4 {
                self.rs_idx_by_missing.shrink_to_fit();
            }

            return true;
        }

        // Either we already had it, or we never wanted it at all.
        false
    }

    /// Construct a (possibly invalid) Relay object from a routerstatus and its
    /// position within the consensus.
    fn relay_from_rs_and_idx<'a>(
        &'a self,
        rs: &'a netstatus::MdConsensusRouterStatus,
        rs_idx: usize,
    ) -> UncheckedRelay<'a> {
        debug_assert_eq!(
            self.consensus.relays()[rs_idx].rsa_identity(),
            rs.rsa_identity()
        );
        let md = self.mds[rs_idx].as_deref();
        if let Some(md) = md {
            debug_assert_eq!(rs.md_digest(), md.digest());
        }

        UncheckedRelay { rs, md }
    }

    /// Replace the overridden parameters in this netdir with `new_replacement`.
    ///
    /// After this function is done, the netdir's parameters will be those in
    /// the consensus, overridden by settings from `new_replacement`.  Any
    /// settings in the old replacement parameters will be discarded.
    pub fn replace_overridden_parameters(&mut self, new_replacement: &netstatus::NetParams<i32>) {
        // TODO(nickm): This is largely duplicate code from PartialNetDir::new().
        let mut new_params = NetParameters::default();
        let _ = new_params.saturating_update(self.consensus.params().iter());
        for u in new_params.saturating_update(new_replacement.iter()) {
            warn!("Unrecognized option: override_net_params.{}", u);
        }

        self.params = new_params;
    }

    /// Return an iterator over all Relay objects, including invalid ones
    /// that we can't use.
    pub fn all_relays(&self) -> impl Iterator<Item = UncheckedRelay<'_>> {
        // TODO: I'd like if we could memoize this so we don't have to
        // do so many hashtable lookups.
        self.consensus
            .relays()
            .iter()
            .enumerate()
            .map(move |(idx, rs)| self.relay_from_rs_and_idx(rs, idx))
    }
    /// Return an iterator over all usable Relays.
    pub fn relays(&self) -> impl Iterator<Item = Relay<'_>> {
        self.all_relays().filter_map(UncheckedRelay::into_relay)
    }

    /// Return a relay matching a given identity, if we have a
    /// _usable_ relay with that key.
    ///
    /// (Does not return unusable relays.)
    ///
    ///    
    /// Note that a `None` answer is not always permanent: if a microdescriptor
    /// is subsequently added for a relay with this ID, the ID may become usable
    /// even if it was not usable before.
    #[allow(clippy::missing_panics_doc)] // Can't panic on valid object.
    pub fn by_id<'a, T>(&self, id: T) -> Option<Relay<'_>>
    where
        T: Into<RelayIdRef<'a>> + ?Sized,
    {
        let id = id.into();
        let answer = match id {
            RelayIdRef::Ed25519(ed25519) => {
                let rs_idx = *self.rs_idx_by_ed.get(ed25519)?;
                let rs = self.consensus.relays().get(rs_idx).expect("Corrupt index");

                self.relay_from_rs_and_idx(rs, rs_idx).into_relay()?
            }
            RelayIdRef::Rsa(rsa) => self
                .by_rsa_id_unchecked(rsa)
                .and_then(UncheckedRelay::into_relay)?,
            other_type => self.relays().find(|r| r.has_identity(other_type))?,
        };
        assert!(answer.has_identity(id));
        Some(answer)
    }

    /// Return a relay with the same identities as those in `target`, if one
    /// exists.
    ///
    /// Does not return unusable relays.
    ///
    /// # Limitations
    ///
    /// This will be very slow if `target` does not have an Ed25519 or RSA
    /// identity.
    pub fn by_ids<T>(&self, target: &T) -> Option<Relay<'_>>
    where
        T: HasRelayIds + ?Sized,
    {
        let mut identities = target.identities();
        // Don't try if there are no identities.
        let first_id = identities.next()?;

        // Since there is at most one relay with each given ID type,
        // we only need to check the first relay we find.
        let candidate = self.by_id(first_id)?;
        if identities.all(|wanted_id| candidate.has_identity(wanted_id)) {
            Some(candidate)
        } else {
            None
        }
    }

    /// Return a boolean if this consensus definitely has (or does not have) a
    /// relay matching the listed identities.
    ///
    ///
    /// If we can't yet tell for sure, return None. Once function has returned
    /// `Some(b)`, it will always return that value for the same `ed_id` and
    /// `rsa_id` on this `NetDir`.  A `None` answer may later become `Some(b)`
    /// if a microdescriptor arrives.
    fn id_pair_listed(&self, ed_id: &Ed25519Identity, rsa_id: &RsaIdentity) -> Option<bool> {
        let r = self.by_rsa_id_unchecked(rsa_id);
        match r {
            Some(unchecked) => {
                if !unchecked.rs.ed25519_id_is_usable() {
                    return Some(false);
                }
                // If md is present, then it's listed iff we have the right
                // ed id.  Otherwise we don't know if it's listed.
                unchecked.md.map(|md| md.ed25519_id() == ed_id)
            }
            None => {
                // Definitely not listed.
                Some(false)
            }
        }
    }

    /// As `id_pair_listed`, but check whether a relay exists (or may exist)
    /// with the same identities as those in `target`.
    ///
    /// # Limitations
    ///
    /// This can be inefficient if the target does not have both an ed25519 and
    /// an rsa identity key.
    pub fn ids_listed<T>(&self, target: &T) -> Option<bool>
    where
        T: HasRelayIds + ?Sized,
    {
        let rsa_id = target.rsa_identity();
        let ed25519_id = target.ed_identity();

        // TODO: If we later support more identity key types, this will
        // become incorrect.  This assertion might help us recognize that case.
        const_assert!(RelayIdType::COUNT == 2);

        match (rsa_id, ed25519_id) {
            (Some(r), Some(e)) => self.id_pair_listed(e, r),
            (Some(r), None) => Some(self.rsa_id_is_listed(r)),
            (None, Some(e)) => {
                if self.rs_idx_by_ed.contains_key(e) {
                    Some(true)
                } else {
                    None
                }
            }
            (None, None) => None,
        }
    }

    /// Return a (possibly unusable) relay with a given RSA identity.
    #[allow(clippy::missing_panics_doc)] // Can't panic on valid object.
    fn by_rsa_id_unchecked(&self, rsa_id: &RsaIdentity) -> Option<UncheckedRelay<'_>> {
        let rs_idx = *self.rs_idx_by_rsa.get(rsa_id)?;
        let rs = self.consensus.relays().get(rs_idx).expect("Corrupt index");
        assert_eq!(rs.rsa_identity(), rsa_id);
        Some(self.relay_from_rs_and_idx(rs, rs_idx))
    }
    /// Return the relay with a given RSA identity, if we have one
    /// and it is usable.
    fn by_rsa_id(&self, rsa_id: &RsaIdentity) -> Option<Relay<'_>> {
        self.by_rsa_id_unchecked(rsa_id)?.into_relay()
    }
    /// Return true if `rsa_id` is listed in this directory, even if it
    /// isn't currently usable.
    fn rsa_id_is_listed(&self, rsa_id: &RsaIdentity) -> bool {
        self.by_rsa_id_unchecked(rsa_id).is_some()
    }

    /// Return the parameters from the consensus, clamped to the
    /// correct ranges, with defaults filled in.
    ///
    /// NOTE: that unsupported parameters aren't returned here; only those
    /// values configured in the `params` module are available.
    pub fn params(&self) -> &NetParameters {
        &self.params
    }
    /// Return weighted the fraction of relays we can use.  We only
    /// consider relays that match the predicate `usable`.  We weight
    /// this bandwidth according to the provided `role`.
    ///
    /// If _no_ matching relays in the consensus have a nonzero
    /// weighted bandwidth value, we fall back to looking at the
    /// unweighted fraction of matching relays.
    ///
    /// If there are no matching relays in the consensus, we return 0.0.
    fn frac_for_role<'a, F>(&'a self, role: WeightRole, usable: F) -> f64
    where
        F: Fn(&UncheckedRelay<'a>) -> bool,
    {
        let mut total_weight = 0_u64;
        let mut have_weight = 0_u64;
        let mut have_count = 0_usize;
        let mut total_count = 0_usize;

        for r in self.all_relays() {
            if !usable(&r) {
                continue;
            }
            let w = self.weights.weight_rs_for_role(r.rs, role);
            total_weight += w;
            total_count += 1;
            if r.is_usable() {
                have_weight += w;
                have_count += 1;
            }
        }

        if total_weight > 0 {
            // The consensus lists some weighted bandwidth so return the
            // fraction of the weighted bandwidth for which we have
            // descriptors.
            (have_weight as f64) / (total_weight as f64)
        } else if total_count > 0 {
            // The consensus lists no weighted bandwidth for these relays,
            // but at least it does list relays. Return the fraction of
            // relays for which it we have descriptors.
            (have_count as f64) / (total_count as f64)
        } else {
            // There are no relays of this kind in the consensus.  Return
            // 0.0, to avoid dividing by zero and giving NaN.
            0.0
        }
    }
    /// Return the estimated fraction of possible paths that we have
    /// enough microdescriptors to build.
    fn frac_usable_paths(&self) -> f64 {
        let f_g = self.frac_for_role(WeightRole::Guard, |u| u.rs.is_flagged_guard());
        let f_m = self.frac_for_role(WeightRole::Middle, |_| true);
        let f_e = if self.all_relays().any(|u| u.rs.is_flagged_exit()) {
            self.frac_for_role(WeightRole::Exit, |u| u.rs.is_flagged_exit())
        } else {
            // If there are no exits at all, we use f_m here.
            f_m
        };
        f_g * f_m * f_e
    }
    /// Return true if there is enough information in this NetDir to build
    /// multihop circuits.

    fn have_enough_paths(&self) -> bool {
        // TODO-A001: This should check for our guards as well, and
        // make sure that if they're listed in the consensus, we have
        // the descriptors for them.

        // If we can build a randomly chosen path with at least this
        // probability, we know enough information to participate
        // on the network.

        let min_frac_paths: f64 = self.params().min_circuit_path_threshold.as_fraction();

        // What fraction of paths can we build?
        let available = self.frac_usable_paths();

        available >= min_frac_paths
    }
    /// Choose a relay at random.
    ///
    /// Each relay is chosen with probability proportional to its weight
    /// in the role `role`, and is only selected if the predicate `usable`
    /// returns true for it.
    ///
    /// This function returns None if (and only if) there are no relays
    /// with nonzero weight where `usable` returned true.
    pub fn pick_relay<'a, R, P>(
        &'a self,
        rng: &mut R,
        role: WeightRole,
        usable: P,
    ) -> Option<Relay<'a>>
    where
        R: rand::Rng,
        P: FnMut(&Relay<'a>) -> bool,
    {
        use rand::seq::SliceRandom;
        let relays: Vec<_> = self.relays().filter(usable).collect();
        // This algorithm uses rand::distributions::WeightedIndex, and uses
        // gives O(n) time and space  to build the index, plus O(log n)
        // sampling time.
        //
        // We might be better off building a WeightedIndex in advance
        // for each `role`, and then sampling it repeatedly until we
        // get a relay that satisfies `usable`.  Or we might not --
        // that depends heavily on the actual particulars of our
        // inputs.  We probably shouldn't make any changes there
        // unless profiling tells us that this function is in a hot
        // path.
        //
        // The C Tor sampling implementation goes through some trouble
        // here to try to make its path selection constant-time.  I
        // believe that there is no actual remotely exploitable
        // side-channel here however.  It could be worth analyzing in
        // the future.
        //
        // This code will give the wrong result if the total of all weights
        // can exceed u64::MAX.  We make sure that can't happen when we
        // set up `self.weights`.
        relays[..]
            .choose_weighted(rng, |r| self.weights.weight_rs_for_role(r.rs, role))
            .ok()
            .cloned()
    }

    /// Choose `n` relay at random.
    ///
    /// Each relay is chosen with probability proportional to its weight
    /// in the role `role`, and is only selected if the predicate `usable`
    /// returns true for it.
    ///
    /// Relays are chosen without replacement: no relay will be
    /// returned twice. Therefore, the resulting vector may be smaller
    /// than `n` if we happen to have fewer than `n` appropriate relays.
    ///
    /// This function returns an empty vector if (and only if) there
    /// are no relays with nonzero weight where `usable` returned
    /// true.
    pub fn pick_n_relays<'a, R, P>(
        &'a self,
        rng: &mut R,
        n: usize,
        role: WeightRole,
        usable: P,
    ) -> Vec<Relay<'a>>
    where
        R: rand::Rng,
        P: FnMut(&Relay<'a>) -> bool,
    {
        use rand::seq::SliceRandom;
        let relays: Vec<_> = self.relays().filter(usable).collect();
        // NOTE: See discussion in pick_relay().
        let mut relays = match relays[..].choose_multiple_weighted(rng, n, |r| {
            self.weights.weight_rs_for_role(r.rs, role) as f64
        }) {
            Err(_) => Vec::new(),
            Ok(iter) => iter.map(Relay::clone).collect(),
        };
        relays.shuffle(rng);
        relays
    }

    /// Compute the weight with which `relay` will be selected for a given
    /// `role`.
    pub fn relay_weight<'a>(&'a self, relay: &Relay<'a>, role: WeightRole) -> RelayWeight {
        RelayWeight(self.weights.weight_rs_for_role(relay.rs, role))
    }

    /// Compute the total weight with which any relay matching `usable`
    /// will be selected for a given `role`.
    ///
    /// Note: because this function is used to assess the total
    /// properties of the consensus, the `usable` predicate takes a
    /// [`RouterStatus`] rather than a [`Relay`].
    pub fn total_weight<P>(&self, role: WeightRole, usable: P) -> RelayWeight
    where
        P: Fn(&UncheckedRelay<'_>) -> bool,
    {
        self.all_relays()
            .filter_map(|unchecked| {
                if usable(&unchecked) {
                    Some(RelayWeight(
                        self.weights.weight_rs_for_role(unchecked.rs, role),
                    ))
                } else {
                    None
                }
            })
            .sum()
    }

    /// Compute the weight with which a relay with ID `rsa_id` would be
    /// selected for a given `role`.
    ///
    /// Note that weight returned by this function assumes that the
    /// relay with that ID is actually usable; if it isn't usable,
    /// then other weight-related functions will call its weight zero.
    pub fn weight_by_rsa_id(&self, rsa_id: &RsaIdentity, role: WeightRole) -> Option<RelayWeight> {
        self.by_rsa_id_unchecked(rsa_id)
            .map(|unchecked| RelayWeight(self.weights.weight_rs_for_role(unchecked.rs, role)))
    }

    /// Return all relays in this NetDir known to be in the same family as
    /// `relay`.
    ///
    /// This list of members will **not** necessarily include `relay` itself.
    ///
    /// # Limitations
    ///
    /// Two relays only belong to the same family if _each_ relay
    /// claims to share a family with the other.  But if we are
    /// missing a microdescriptor for one of the relays listed by this
    /// relay, we cannot know whether it acknowledges family
    /// membership with this relay or not.  Therefore, this function
    /// can omit family members for which there is not (as yet) any
    /// Relay object.
    pub fn known_family_members<'a>(
        &'a self,
        relay: &'a Relay<'a>,
    ) -> impl Iterator<Item = Relay<'a>> {
        let relay_rsa_id = relay.rsa_id();
        relay.md.family().members().filter_map(move |other_rsa_id| {
            self.by_rsa_id(other_rsa_id)
                .filter(|other_relay| other_relay.md.family().contains(relay_rsa_id))
        })
    }
}

impl MdReceiver for NetDir {
    fn missing_microdescs(&self) -> Box<dyn Iterator<Item = &MdDigest> + '_> {
        Box::new(self.rs_idx_by_missing.keys())
    }
    fn add_microdesc(&mut self, md: Microdesc) -> bool {
        self.add_arc_microdesc(Arc::new(md))
    }
    fn n_missing(&self) -> usize {
        self.rs_idx_by_missing.len()
    }
}

impl<'a> UncheckedRelay<'a> {
    /// Return true if this relay is valid and usable.
    ///
    /// This function should return `true` for every Relay we expose
    /// to the user.
    pub fn is_usable(&self) -> bool {
        // No need to check for 'valid' or 'running': they are implicit.
        self.md.is_some() && self.rs.ed25519_id_is_usable()
    }
    /// If this is usable, return a corresponding Relay object.
    pub fn into_relay(self) -> Option<Relay<'a>> {
        if self.is_usable() {
            Some(Relay {
                rs: self.rs,
                md: self.md?,
            })
        } else {
            None
        }
    }
    /// Return true if this relay has the guard flag.
    pub fn is_flagged_guard(&self) -> bool {
        self.rs.is_flagged_guard()
    }
    /// Return true if this relay is a potential directory cache.
    pub fn is_dir_cache(&self) -> bool {
        rs_is_dir_cache(self.rs)
    }
}

impl<'a> Relay<'a> {
    /// Return the Ed25519 ID for this relay.
    pub fn id(&self) -> &Ed25519Identity {
        self.md.ed25519_id()
    }
    /// Return the RsaIdentity for this relay.
    pub fn rsa_id(&self) -> &RsaIdentity {
        self.rs.rsa_identity()
    }
    /// Return true if this relay and `other` seem to be the same relay.
    ///
    /// (Two relays are the same if they have the same identity.)
    pub fn same_relay<'b>(&self, other: &Relay<'b>) -> bool {
        self.id() == other.id() && self.rsa_id() == other.rsa_id()
    }
    /// Return true if this relay allows exiting to `port` on IPv4.
    pub fn supports_exit_port_ipv4(&self, port: u16) -> bool {
        self.ipv4_policy().allows_port(port)
    }
    /// Return true if this relay allows exiting to `port` on IPv6.
    pub fn supports_exit_port_ipv6(&self, port: u16) -> bool {
        self.ipv6_policy().allows_port(port)
    }
    /// Return true if this relay is suitable for use as a directory
    /// cache.
    pub fn is_dir_cache(&self) -> bool {
        rs_is_dir_cache(self.rs)
    }
    /// Return true if this relay is marked as usable as a new Guard node.
    pub fn is_flagged_guard(&self) -> bool {
        self.rs.is_flagged_guard()
    }
    /// Return true if both relays are in the same subnet, as configured by
    /// `subnet_config`.
    ///
    /// Two relays are considered to be in the same subnet if they
    /// have IPv4 addresses with the same `subnets_family_v4`-bit
    /// prefix, or if they have IPv6 addresses with the same
    /// `subnets_family_v6`-bit prefix.
    pub fn in_same_subnet<'b>(&self, other: &Relay<'b>, subnet_config: &SubnetConfig) -> bool {
        self.rs.orport_addrs().any(|addr| {
            other
                .rs
                .orport_addrs()
                .any(|other| subnet_config.addrs_in_same_subnet(&addr.ip(), &other.ip()))
        })
    }
    /// Return true if both relays are in the same family.
    ///
    /// (Every relay is considered to be in the same family as itself.)
    pub fn in_same_family<'b>(&self, other: &Relay<'b>) -> bool {
        if self.same_relay(other) {
            return true;
        }
        self.md.family().contains(other.rsa_id()) && other.md.family().contains(self.rsa_id())
    }

    /// Return true if there are any ports for which this Relay can be
    /// used for exit traffic.
    ///
    /// (Returns false if this relay doesn't allow exit traffic, or if it
    /// has been flagged as a bad exit.)
    pub fn policies_allow_some_port(&self) -> bool {
        if self.rs.is_flagged_bad_exit() {
            return false;
        }

        self.md.ipv4_policy().allows_some_port() || self.md.ipv6_policy().allows_some_port()
    }

    /// Return the IPv4 exit policy for this relay. If the relay has been marked BadExit, return an
    /// empty policy
    pub fn ipv4_policy(&self) -> Arc<PortPolicy> {
        if !self.rs.is_flagged_bad_exit() {
            Arc::clone(self.md.ipv4_policy())
        } else {
            Arc::new(PortPolicy::new_reject_all())
        }
    }
    /// Return the IPv6 exit policy for this relay. If the relay has been marked BadExit, return an
    /// empty policy
    pub fn ipv6_policy(&self) -> Arc<PortPolicy> {
        if !self.rs.is_flagged_bad_exit() {
            Arc::clone(self.md.ipv6_policy())
        } else {
            Arc::new(PortPolicy::new_reject_all())
        }
    }
    /// Return the IPv4 exit policy declared by this relay. Contrary to [`Relay::ipv4_policy`],
    /// this does not verify if the relay is marked BadExit.
    pub fn ipv4_declared_policy(&self) -> &Arc<PortPolicy> {
        self.md.ipv4_policy()
    }
    /// Return the IPv6 exit policy declared by this relay. Contrary to [`Relay::ipv6_policy`],
    /// this does not verify if the relay is marked BadExit.
    pub fn ipv6_declared_policy(&self) -> &Arc<PortPolicy> {
        self.md.ipv6_policy()
    }

    /// Return a reference to this relay's "router status" entry in
    /// the consensus.
    ///
    /// The router status entry contains information about the relay
    /// that the authorities voted on directly.  For most use cases,
    /// you shouldn't need them.
    ///
    /// This function is only available if the crate was built with
    /// its `experimental-api` feature.
    #[cfg(feature = "experimental-api")]
    pub fn rs(&self) -> &netstatus::MdConsensusRouterStatus {
        self.rs
    }
    /// Return a reference to this relay's "microdescriptor" entry in
    /// the consensus.
    ///
    /// A "microdescriptor" is a synopsis of the information about a relay,
    /// used to determine its capabilities and route traffic through it.
    /// For most use cases, you shouldn't need it.
    ///
    /// This function is only available if the crate was built with
    /// its `experimental-api` feature.
    #[cfg(feature = "experimental-api")]
    pub fn md(&self) -> &Microdesc {
        self.md
    }
}

impl<'a> HasAddrs for Relay<'a> {
    fn addrs(&self) -> &[std::net::SocketAddr] {
        self.rs.addrs()
    }
}
impl<'a> tor_linkspec::HasRelayIdsLegacy for Relay<'a> {
    fn ed_identity(&self) -> &Ed25519Identity {
        self.id()
    }
    fn rsa_identity(&self) -> &RsaIdentity {
        self.rsa_id()
    }
}

impl<'a> HasRelayIds for UncheckedRelay<'a> {
    fn identity(&self, key_type: RelayIdType) -> Option<RelayIdRef<'_>> {
        match key_type {
            RelayIdType::Ed25519 if self.rs.ed25519_id_is_usable() => {
                self.md.map(|m| m.ed25519_id().into())
            }
            RelayIdType::Rsa => Some(self.rs.rsa_identity().into()),
            _ => None,
        }
    }
}

impl<'a> ChanTarget for Relay<'a> {}

impl<'a> tor_linkspec::CircTarget for Relay<'a> {
    fn ntor_onion_key(&self) -> &ll::pk::curve25519::PublicKey {
        self.md.ntor_key()
    }
    fn protovers(&self) -> &tor_protover::Protocols {
        self.rs.protovers()
    }
}

/// Return true if `rs` is usable as a directory cache.
fn rs_is_dir_cache(rs: &netstatus::MdConsensusRouterStatus) -> bool {
    use tor_protover::ProtoKind;
    rs.is_flagged_v2dir() && rs.protovers().supports_known_subver(ProtoKind::DirCache, 2)
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::cognitive_complexity)]
    use super::*;
    use crate::testnet::*;
    use float_eq::assert_float_eq;
    use std::collections::HashSet;
    use std::time::Duration;
    use tor_basic_utils::test_rng;
    use tor_linkspec::{RelayIdType, RelayIds};

    // Basic functionality for a partial netdir: Add microdescriptors,
    // then you have a netdir.
    #[test]
    fn partial_netdir() {
        let (consensus, microdescs) = construct_network().unwrap();
        let dir = PartialNetDir::new(consensus, None);

        // Check the lifetime
        let lifetime = dir.lifetime();
        assert_eq!(
            lifetime
                .valid_until()
                .duration_since(lifetime.valid_after())
                .unwrap(),
            Duration::new(86400, 0)
        );

        // No microdescriptors, so we don't have enough paths, and can't
        // advance.
        assert!(!dir.have_enough_paths());
        let mut dir = match dir.unwrap_if_sufficient() {
            Ok(_) => panic!(),
            Err(d) => d,
        };

        let missing: HashSet<_> = dir.missing_microdescs().collect();
        assert_eq!(missing.len(), 40);
        assert_eq!(missing.len(), dir.netdir.consensus.relays().len());
        for md in &microdescs {
            assert!(missing.contains(md.digest()));
        }

        // Now add all the mds and try again.
        for md in microdescs {
            let wanted = dir.add_microdesc(md);
            assert!(wanted);
        }

        let missing: HashSet<_> = dir.missing_microdescs().collect();
        assert!(missing.is_empty());
        assert!(dir.have_enough_paths());
        let _complete = match dir.unwrap_if_sufficient() {
            Ok(d) => d,
            Err(_) => panic!(),
        };
    }

    #[test]
    fn override_params() {
        let (consensus, _microdescs) = construct_network().unwrap();
        let override_p = "bwweightscale=2 doesnotexist=77 circwindow=500"
            .parse()
            .unwrap();
        let dir = PartialNetDir::new(consensus.clone(), Some(&override_p));
        let params = &dir.netdir.params;
        assert_eq!(params.bw_weight_scale.get(), 2);
        assert_eq!(params.circuit_window.get(), 500_i32);

        // try again without the override.
        let dir = PartialNetDir::new(consensus, None);
        let params = &dir.netdir.params;
        assert_eq!(params.bw_weight_scale.get(), 1_i32);
        assert_eq!(params.circuit_window.get(), 1000_i32);
    }

    #[test]
    fn fill_from_previous() {
        let (consensus, microdescs) = construct_network().unwrap();

        let mut dir = PartialNetDir::new(consensus.clone(), None);
        for md in microdescs.iter().skip(2) {
            let wanted = dir.add_microdesc(md.clone());
            assert!(wanted);
        }
        let dir1 = dir.unwrap_if_sufficient().unwrap();
        assert_eq!(dir1.missing_microdescs().count(), 2);

        let mut dir = PartialNetDir::new(consensus, None);
        assert_eq!(dir.missing_microdescs().count(), 40);
        dir.fill_from_previous_netdir(&dir1);
        assert_eq!(dir.missing_microdescs().count(), 2);
    }

    #[test]
    fn path_count() {
        let low_threshold = "min_paths_for_circs_pct=64".parse().unwrap();
        let high_threshold = "min_paths_for_circs_pct=65".parse().unwrap();

        let (consensus, microdescs) = construct_network().unwrap();

        let mut dir = PartialNetDir::new(consensus.clone(), Some(&low_threshold));
        for (idx, md) in microdescs.iter().enumerate() {
            if idx % 7 == 2 {
                continue; // skip a few relays.
            }
            dir.add_microdesc(md.clone());
        }
        let dir = dir.unwrap_if_sufficient().unwrap();

        // We  have 40 relays that we know about from the consensus.
        assert_eq!(dir.all_relays().count(), 40);

        // But only 34 are usable.
        assert_eq!(dir.relays().count(), 34);

        // For guards: mds 20..=39 correspond to Guard relays.
        // Their bandwidth is 2*(1000+2000+...10000) = 110_000.
        // We skipped 23, 30, and 37.  They have bandwidth
        // 4000 + 1000 + 8000 = 13_000.  So our fractional bandwidth
        // should be (110-13)/110.
        let f = dir.frac_for_role(WeightRole::Guard, |u| u.rs.is_flagged_guard());
        assert!(((97.0 / 110.0) - f).abs() < 0.000001);

        // For exits: mds 10..=19 and 30..=39 correspond to Exit relays.
        // We skipped 16, 30,  and 37. Per above our fractional bandwidth is
        // (110-16)/110.
        let f = dir.frac_for_role(WeightRole::Exit, |u| u.rs.is_flagged_exit());
        assert!(((94.0 / 110.0) - f).abs() < 0.000001);

        // For middles: all relays are middles. We skipped 2, 9, 16,
        // 23, 30, and 37. Per above our fractional bandwidth is
        // (220-33)/220
        let f = dir.frac_for_role(WeightRole::Middle, |_| true);
        assert!(((187.0 / 220.0) - f).abs() < 0.000001);

        // Multiplying those together, we get the fraction of paths we can
        // build at ~0.64052066, which is above the threshold we set above for
        // MinPathsForCircsPct.
        let f = dir.frac_usable_paths();
        assert!((f - 0.64052066).abs() < 0.000001);

        // But if we try again with a slightly higher threshold...
        let mut dir = PartialNetDir::new(consensus, Some(&high_threshold));
        for (idx, md) in microdescs.into_iter().enumerate() {
            if idx % 7 == 2 {
                continue; // skip a few relays.
            }
            dir.add_microdesc(md);
        }
        assert!(dir.unwrap_if_sufficient().is_err());
    }

    /// Return a 3-tuple for use by `test_pick_*()` of an Rng, a number of
    /// iterations, and a tolerance.
    ///
    /// If the Rng is deterministic (the default), we can use a faster setup,
    /// with a higher tolerance and fewer iterations.  But if you've explicitly
    /// opted into randomization (or are replaying a seed from an earlier
    /// randomized test), we give you more iterations and a tighter tolerance.
    fn testing_rng_with_tolerances() -> (impl rand::Rng, usize, f64) {
        // Use a deterministic RNG if none is specified, since this is slow otherwise.
        let config = test_rng::Config::from_env().unwrap_or(test_rng::Config::Deterministic);
        let (iters, tolerance) = match config {
            test_rng::Config::Deterministic => (5000, 0.02),
            _ => (50000, 0.01),
        };
        (config.into_rng(), iters, tolerance)
    }

    #[test]
    fn test_pick() {
        let (consensus, microdescs) = construct_network().unwrap();
        let mut dir = PartialNetDir::new(consensus, None);
        for md in microdescs.into_iter() {
            let wanted = dir.add_microdesc(md.clone());
            assert!(wanted);
        }
        let dir = dir.unwrap_if_sufficient().unwrap();

        let (mut rng, total, tolerance) = testing_rng_with_tolerances();

        let mut picked = [0_isize; 40];
        for _ in 0..total {
            let r = dir.pick_relay(&mut rng, WeightRole::Middle, |r| {
                r.supports_exit_port_ipv4(80)
            });
            let r = r.unwrap();
            let id_byte = r.identity(RelayIdType::Rsa).unwrap().as_bytes()[0];
            picked[id_byte as usize] += 1;
        }
        // non-exits should never get picked.
        picked[0..10].iter().for_each(|x| assert_eq!(*x, 0));
        picked[20..30].iter().for_each(|x| assert_eq!(*x, 0));

        let picked_f: Vec<_> = picked.iter().map(|x| *x as f64 / total as f64).collect();

        // We didn't we any non-default weights, so the other relays get
        // weighted proportional to their bandwidth.
        assert_float_eq!(picked_f[19], (10.0 / 110.0), abs <= tolerance);
        assert_float_eq!(picked_f[38], (9.0 / 110.0), abs <= tolerance);
        assert_float_eq!(picked_f[39], (10.0 / 110.0), abs <= tolerance);
    }

    #[test]
    fn test_pick_multiple() {
        // This is mostly a copy of test_pick, except that it uses
        // pick_n_relays to pick several relays at once.

        let dir = construct_netdir().unwrap_if_sufficient().unwrap();

        let (mut rng, total, tolerance) = testing_rng_with_tolerances();

        let mut picked = [0_isize; 40];
        for _ in 0..total / 4 {
            let relays = dir.pick_n_relays(&mut rng, 4, WeightRole::Middle, |r| {
                r.supports_exit_port_ipv4(80)
            });
            assert_eq!(relays.len(), 4);
            for r in relays {
                let id_byte = r.identity(RelayIdType::Rsa).unwrap().as_bytes()[0];
                picked[id_byte as usize] += 1;
            }
        }
        // non-exits should never get picked.
        picked[0..10].iter().for_each(|x| assert_eq!(*x, 0));
        picked[20..30].iter().for_each(|x| assert_eq!(*x, 0));

        let picked_f: Vec<_> = picked.iter().map(|x| *x as f64 / total as f64).collect();

        // We didn't we any non-default weights, so the other relays get
        // weighted proportional to their bandwidth.
        assert_float_eq!(picked_f[19], (10.0 / 110.0), abs <= tolerance);
        assert_float_eq!(picked_f[36], (7.0 / 110.0), abs <= tolerance);
        assert_float_eq!(picked_f[39], (10.0 / 110.0), abs <= tolerance);
    }

    #[test]
    fn subnets() {
        let cfg = SubnetConfig::default();

        fn same_net(cfg: &SubnetConfig, a: &str, b: &str) -> bool {
            cfg.addrs_in_same_subnet(&a.parse().unwrap(), &b.parse().unwrap())
        }

        assert!(same_net(&cfg, "127.15.3.3", "127.15.9.9"));
        assert!(!same_net(&cfg, "127.15.3.3", "127.16.9.9"));

        assert!(!same_net(&cfg, "127.15.3.3", "127::"));

        assert!(same_net(&cfg, "ffff:ffff:90:33::", "ffff:ffff:91:34::"));
        assert!(!same_net(&cfg, "ffff:ffff:90:33::", "ffff:fffe:91:34::"));

        let cfg = SubnetConfig {
            subnets_family_v4: 32,
            subnets_family_v6: 128,
        };
        assert!(!same_net(&cfg, "127.15.3.3", "127.15.9.9"));
        assert!(!same_net(&cfg, "ffff:ffff:90:33::", "ffff:ffff:91:34::"));

        assert!(same_net(&cfg, "127.0.0.1", "127.0.0.1"));
        assert!(!same_net(&cfg, "127.0.0.1", "127.0.0.2"));
        assert!(same_net(&cfg, "ffff:ffff:90:33::", "ffff:ffff:90:33::"));

        let cfg = SubnetConfig {
            subnets_family_v4: 33,
            subnets_family_v6: 129,
        };
        assert!(!same_net(&cfg, "127.0.0.1", "127.0.0.1"));
        assert!(!same_net(&cfg, "::", "::"));
    }

    #[test]
    fn relay_funcs() {
        let (consensus, microdescs) = construct_custom_network(|idx, nb| {
            if idx == 15 {
                nb.rs.add_or_port("[f0f0::30]:9001".parse().unwrap());
            } else if idx == 20 {
                nb.rs.add_or_port("[f0f0::3131]:9001".parse().unwrap());
            }
        })
        .unwrap();
        let subnet_config = SubnetConfig::default();
        let mut dir = PartialNetDir::new(consensus, None);
        for md in microdescs.into_iter() {
            let wanted = dir.add_microdesc(md.clone());
            assert!(wanted);
        }
        let dir = dir.unwrap_if_sufficient().unwrap();

        // Pick out a few relays by ID.
        let k0 = Ed25519Identity::from([0; 32]);
        let k1 = Ed25519Identity::from([1; 32]);
        let k2 = Ed25519Identity::from([2; 32]);
        let k3 = Ed25519Identity::from([3; 32]);
        let k10 = Ed25519Identity::from([10; 32]);
        let k15 = Ed25519Identity::from([15; 32]);
        let k20 = Ed25519Identity::from([20; 32]);

        let r0 = dir.by_id(&k0).unwrap();
        let r1 = dir.by_id(&k1).unwrap();
        let r2 = dir.by_id(&k2).unwrap();
        let r3 = dir.by_id(&k3).unwrap();
        let r10 = dir.by_id(&k10).unwrap();
        let r15 = dir.by_id(&k15).unwrap();
        let r20 = dir.by_id(&k20).unwrap();

        assert_eq!(r0.id(), &[0; 32].into());
        assert_eq!(r0.rsa_id(), &[0; 20].into());
        assert_eq!(r1.id(), &[1; 32].into());
        assert_eq!(r1.rsa_id(), &[1; 20].into());

        assert!(r0.same_relay(&r0));
        assert!(r1.same_relay(&r1));
        assert!(!r1.same_relay(&r0));

        assert!(r0.is_dir_cache());
        assert!(!r1.is_dir_cache());
        assert!(r2.is_dir_cache());
        assert!(!r3.is_dir_cache());

        assert!(!r0.supports_exit_port_ipv4(80));
        assert!(!r1.supports_exit_port_ipv4(80));
        assert!(!r2.supports_exit_port_ipv4(80));
        assert!(!r3.supports_exit_port_ipv4(80));

        assert!(!r0.policies_allow_some_port());
        assert!(!r1.policies_allow_some_port());
        assert!(!r2.policies_allow_some_port());
        assert!(!r3.policies_allow_some_port());
        assert!(r10.policies_allow_some_port());

        assert!(r0.in_same_family(&r0));
        assert!(r0.in_same_family(&r1));
        assert!(r1.in_same_family(&r0));
        assert!(r1.in_same_family(&r1));
        assert!(!r0.in_same_family(&r2));
        assert!(!r2.in_same_family(&r0));
        assert!(r2.in_same_family(&r2));
        assert!(r2.in_same_family(&r3));

        assert!(r0.in_same_subnet(&r10, &subnet_config));
        assert!(r10.in_same_subnet(&r10, &subnet_config));
        assert!(r0.in_same_subnet(&r0, &subnet_config));
        assert!(r1.in_same_subnet(&r1, &subnet_config));
        assert!(!r1.in_same_subnet(&r2, &subnet_config));
        assert!(!r2.in_same_subnet(&r3, &subnet_config));

        // Make sure IPv6 families work.
        let subnet_config = SubnetConfig {
            subnets_family_v4: 128,
            subnets_family_v6: 96,
        };
        assert!(r15.in_same_subnet(&r20, &subnet_config));
        assert!(!r15.in_same_subnet(&r1, &subnet_config));

        // Make sure that subnet configs can be disabled.
        let subnet_config = SubnetConfig {
            subnets_family_v4: 255,
            subnets_family_v6: 255,
        };
        assert!(!r15.in_same_subnet(&r20, &subnet_config));
    }

    #[test]
    fn test_badexit() {
        // make a netdir where relays 10-19 are badexit, and everybody
        // exits to 443 on IPv6.
        use tor_netdoc::doc::netstatus::RelayFlags;
        let netdir = construct_custom_netdir(|idx, nb| {
            if (10..20).contains(&idx) {
                nb.rs.add_flags(RelayFlags::BAD_EXIT);
            }
            nb.md.parse_ipv6_policy("accept 443").unwrap();
        })
        .unwrap()
        .unwrap_if_sufficient()
        .unwrap();

        let e12 = netdir.by_id(&Ed25519Identity::from([12; 32])).unwrap();
        let e32 = netdir.by_id(&Ed25519Identity::from([32; 32])).unwrap();

        assert!(!e12.supports_exit_port_ipv4(80));
        assert!(e32.supports_exit_port_ipv4(80));

        assert!(!e12.supports_exit_port_ipv6(443));
        assert!(e32.supports_exit_port_ipv6(443));
        assert!(!e32.supports_exit_port_ipv6(555));

        assert!(!e12.policies_allow_some_port());
        assert!(e32.policies_allow_some_port());

        assert!(!e12.ipv4_policy().allows_some_port());
        assert!(!e12.ipv6_policy().allows_some_port());
        assert!(e32.ipv4_policy().allows_some_port());
        assert!(e32.ipv6_policy().allows_some_port());

        assert!(e12.ipv4_declared_policy().allows_some_port());
        assert!(e12.ipv6_declared_policy().allows_some_port());
    }

    #[cfg(feature = "experimental-api")]
    #[test]
    fn test_accessors() {
        let netdir = construct_netdir().unwrap_if_sufficient().unwrap();

        let r4 = netdir.by_id(&Ed25519Identity::from([4; 32])).unwrap();
        let r16 = netdir.by_id(&Ed25519Identity::from([16; 32])).unwrap();

        assert!(!r4.md().ipv4_policy().allows_some_port());
        assert!(r16.md().ipv4_policy().allows_some_port());

        assert!(!r4.rs().is_flagged_exit());
        assert!(r16.rs().is_flagged_exit());
    }

    #[test]
    fn test_by_id() {
        // Make a netdir that omits the microdescriptor for 0xDDDDDD...
        let netdir = construct_custom_netdir(|idx, mut nb| {
            nb.omit_md = idx == 13;
        })
        .unwrap();

        let netdir = netdir.unwrap_if_sufficient().unwrap();

        let r = netdir.by_id(&Ed25519Identity::from([0; 32])).unwrap();
        assert_eq!(r.id().as_bytes(), &[0; 32]);

        assert!(netdir.by_id(&Ed25519Identity::from([13; 32])).is_none());

        let r = netdir.by_rsa_id(&[12; 20].into()).unwrap();
        assert_eq!(r.rsa_id().as_bytes(), &[12; 20]);
        assert!(netdir.rsa_id_is_listed(&[12; 20].into()));

        assert!(netdir.by_rsa_id(&[13; 20].into()).is_none());

        assert!(netdir.by_rsa_id_unchecked(&[99; 20].into()).is_none());
        assert!(!netdir.rsa_id_is_listed(&[99; 20].into()));

        let r = netdir.by_rsa_id_unchecked(&[13; 20].into()).unwrap();
        assert_eq!(r.rs.rsa_identity().as_bytes(), &[13; 20]);
        assert!(netdir.rsa_id_is_listed(&[13; 20].into()));

        let pair_13_13 = RelayIds::new([13; 32].into(), [13; 20].into());
        let pair_14_14 = RelayIds::new([14; 32].into(), [14; 20].into());
        let pair_14_99 = RelayIds::new([14; 32].into(), [99; 20].into());

        let r = netdir.by_ids(&pair_13_13);
        assert!(r.is_none());
        let r = netdir.by_ids(&pair_14_14).unwrap();
        assert_eq!(r.identity(RelayIdType::Rsa).unwrap().as_bytes(), &[14; 20]);
        assert_eq!(
            r.identity(RelayIdType::Ed25519).unwrap().as_bytes(),
            &[14; 32]
        );
        let r = netdir.by_ids(&pair_14_99);
        assert!(r.is_none());

        assert_eq!(
            netdir.id_pair_listed(&[13; 32].into(), &[13; 20].into()),
            None
        );
        assert_eq!(
            netdir.id_pair_listed(&[15; 32].into(), &[15; 20].into()),
            Some(true)
        );
        assert_eq!(
            netdir.id_pair_listed(&[15; 32].into(), &[99; 20].into()),
            Some(false)
        );
    }

    #[test]
    fn weight_type() {
        let r0 = RelayWeight(0);
        let r100 = RelayWeight(100);
        let r200 = RelayWeight(200);
        let r300 = RelayWeight(300);
        assert_eq!(r100 + r200, r300);
        assert_eq!(r100.checked_div(r200), Some(0.5));
        assert!(r100.checked_div(r0).is_none());
        assert_eq!(r200.ratio(0.5), Some(r100));
        assert!(r200.ratio(-1.0).is_none());
    }

    #[test]
    fn weight_accessors() {
        // Make a netdir that omits the microdescriptor for 0xDDDDDD...
        let netdir = construct_netdir().unwrap_if_sufficient().unwrap();

        let g_total = netdir.total_weight(WeightRole::Guard, |r| r.is_flagged_guard());
        // This is just the total guard weight, since all our Wxy = 1.
        assert_eq!(g_total, RelayWeight(110_000));

        let g_total = netdir.total_weight(WeightRole::Guard, |_| false);
        assert_eq!(g_total, RelayWeight(0));

        let relay = netdir.by_id(&Ed25519Identity::from([35; 32])).unwrap();
        assert!(relay.is_flagged_guard());
        let w = netdir.relay_weight(&relay, WeightRole::Guard);
        assert_eq!(w, RelayWeight(6_000));

        let w = netdir
            .weight_by_rsa_id(&[33; 20].into(), WeightRole::Guard)
            .unwrap();
        assert_eq!(w, RelayWeight(4_000));

        assert!(netdir
            .weight_by_rsa_id(&[99; 20].into(), WeightRole::Guard)
            .is_none());
    }

    #[test]
    fn family_list() {
        let netdir = construct_custom_netdir(|idx, n| {
            if idx == 0x0a {
                n.md.family(
                    "$0B0B0B0B0B0B0B0B0B0B0B0B0B0B0B0B0B0B0B0B \
                     $0C0C0C0C0C0C0C0C0C0C0C0C0C0C0C0C0C0C0C0C \
                     $0D0D0D0D0D0D0D0D0D0D0D0D0D0D0D0D0D0D0D0D"
                        .parse()
                        .unwrap(),
                );
            } else if idx == 0x0c {
                n.md.family("$0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A".parse().unwrap());
            }
        })
        .unwrap()
        .unwrap_if_sufficient()
        .unwrap();

        // In the testing netdir, adjacent members are in the same family by default...
        let r0 = netdir.by_id(&Ed25519Identity::from([0; 32])).unwrap();
        let family: Vec<_> = netdir.known_family_members(&r0).collect();
        assert_eq!(family.len(), 1);
        assert_eq!(family[0].id(), &Ed25519Identity::from([1; 32]));

        // But we've made this relay claim membership with several others.
        let r10 = netdir.by_id(&Ed25519Identity::from([10; 32])).unwrap();
        let family: HashSet<_> = netdir.known_family_members(&r10).map(|r| *r.id()).collect();
        assert_eq!(family.len(), 2);
        assert!(family.contains(&Ed25519Identity::from([11; 32])));
        assert!(family.contains(&Ed25519Identity::from([12; 32])));
        // Note that 13 doesn't get put in, even though it's listed, since it doesn't claim
        //  membership with 10.
    }
}
