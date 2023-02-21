//! Code to construct paths through the Tor network
//!
//! TODO: I'm not sure this belongs in circmgr, but this is the best place
//! I can think of for now.  I'm also not sure this should be public.

pub mod dirpath;
pub mod exitpath;

use tor_error::bad_api_usage;
use tor_guardmgr::fallback::FallbackDir;
use tor_linkspec::{OwnedChanTarget, OwnedCircTarget};
use tor_netdir::Relay;

use crate::usage::ExitPolicy;
use crate::Result;

/// A list of Tor relays through the network.
pub struct TorPath<'a> {
    /// The inner TorPath state.
    inner: TorPathInner<'a>,
}

/// Non-public helper type to represent the different kinds of Tor path.
///
/// (This is a separate type to avoid exposing its details to the user.)
enum TorPathInner<'a> {
    /// A single-hop path for use with a directory cache, when a relay is
    /// known.
    OneHop(Relay<'a>), // This could just be a routerstatus.
    /// A single-hop path for use with a directory cache, when we don't have
    /// a consensus.
    FallbackOneHop(&'a FallbackDir),
    /// A single-hop path taken from an OwnedChanTarget.
    OwnedOneHop(OwnedChanTarget),
    /// A multi-hop path, containing one or more relays.
    Path(Vec<Relay<'a>>),
}

impl<'a> TorPath<'a> {
    /// Create a new one-hop path for use with a directory cache with a known
    /// relay.
    pub fn new_one_hop(relay: Relay<'a>) -> Self {
        Self {
            inner: TorPathInner::OneHop(relay),
        }
    }

    /// Create a new one-hop path for use with a directory cache when we don't
    /// have a consensus.
    pub fn new_fallback_one_hop(fallback_dir: &'a FallbackDir) -> Self {
        Self {
            inner: TorPathInner::FallbackOneHop(fallback_dir),
        }
    }

    /// Construct a new one-hop path for directory use from an arbitrarily
    /// chosen channel target.
    pub fn new_one_hop_owned<T: tor_linkspec::ChanTarget>(target: &T) -> Self {
        Self {
            inner: TorPathInner::OwnedOneHop(OwnedChanTarget::from_chan_target(target)),
        }
    }

    /// Create a new multi-hop path with a given number of ordered relays.
    pub fn new_multihop(relays: impl IntoIterator<Item = Relay<'a>>) -> Self {
        Self {
            inner: TorPathInner::Path(relays.into_iter().collect()),
        }
    }

    /// Return the final relay in this path, if this is a path for use
    /// with exit circuits.
    fn exit_relay(&self) -> Option<&Relay<'a>> {
        match &self.inner {
            TorPathInner::Path(relays) if !relays.is_empty() => Some(&relays[relays.len() - 1]),
            _ => None,
        }
    }

    /// Return the exit policy of the final relay in this path, if this
    /// is a path for use with exit circuits.
    pub(crate) fn exit_policy(&self) -> Option<ExitPolicy> {
        self.exit_relay().map(ExitPolicy::from_relay)
    }

    /// Return the number of relays in this path.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        use TorPathInner::*;
        match &self.inner {
            OneHop(_) => 1,
            FallbackOneHop(_) => 1,
            OwnedOneHop(_) => 1,
            Path(p) => p.len(),
        }
    }
}

/// A path composed entirely of owned components.
#[derive(Clone, Debug)]
pub(crate) enum OwnedPath {
    /// A path where we only know how to make circuits via CREATE_FAST.
    ChannelOnly(OwnedChanTarget),
    /// A path of one or more hops created via normal Tor handshakes.
    Normal(Vec<OwnedCircTarget>),
}

impl<'a> TryFrom<&TorPath<'a>> for OwnedPath {
    type Error = crate::Error;
    fn try_from(p: &TorPath<'a>) -> Result<OwnedPath> {
        use TorPathInner::*;

        Ok(match &p.inner {
            FallbackOneHop(h) => OwnedPath::ChannelOnly(OwnedChanTarget::from_chan_target(*h)),
            OneHop(h) => OwnedPath::Normal(vec![OwnedCircTarget::from_circ_target(h)]),
            OwnedOneHop(owned) => OwnedPath::ChannelOnly(owned.clone()),
            Path(p) if !p.is_empty() => {
                OwnedPath::Normal(p.iter().map(OwnedCircTarget::from_circ_target).collect())
            }
            Path(_) => {
                return Err(bad_api_usage!("Path with no entries!").into());
            }
        })
    }
}

impl OwnedPath {
    /// Return the number of hops in this path.
    #[allow(clippy::len_without_is_empty)]
    pub(crate) fn len(&self) -> usize {
        match self {
            OwnedPath::ChannelOnly(_) => 1,
            OwnedPath::Normal(p) => p.len(),
        }
    }
}

/// For testing: make sure that `path` is the same when it is an owned
/// path.
#[cfg(test)]
fn assert_same_path_when_owned(path: &TorPath<'_>) {
    #![allow(clippy::unwrap_used)]
    use tor_linkspec::HasRelayIds;
    let owned: OwnedPath = path.try_into().unwrap();

    match (&owned, &path.inner) {
        (OwnedPath::ChannelOnly(c), TorPathInner::FallbackOneHop(f)) => {
            assert!(c.same_relay_ids(*f));
        }
        (OwnedPath::Normal(p), TorPathInner::OneHop(h)) => {
            assert_eq!(p.len(), 1);
            assert!(p[0].same_relay_ids(h));
        }
        (OwnedPath::Normal(p1), TorPathInner::Path(p2)) => {
            assert_eq!(p1.len(), p2.len());
            for (n1, n2) in p1.iter().zip(p2.iter()) {
                assert!(n1.same_relay_ids(n2));
            }
        }
        (_, _) => {
            panic!("Mismatched path types.");
        }
    }
}
