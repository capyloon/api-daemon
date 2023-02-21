//! Identifier objects used to specify guards and/or fallbacks.

use derive_more::AsRef;
use serde::{Deserialize, Serialize};
use tor_linkspec::RelayIds;
#[cfg(test)]
use tor_llcrypto::pk;

/// An identifier for a fallback directory cache.
///
/// This is a separate type from GuardId and FirstHopId to avoid confusion
/// about what kind of object we're identifying.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, AsRef)]
pub(crate) struct FallbackId(pub(crate) RelayIds);

impl FallbackId {
    /// Return a new, manually constructed `FallbackId`
    /// Extract a `FallbackId` from a ChanTarget object.
    pub(crate) fn from_relay_ids<T>(target: &T) -> Self
    where
        T: tor_linkspec::HasRelayIds + ?Sized,
    {
        Self(RelayIds::from_relay_ids(target))
    }
}

/// An identifier for a sampled guard.
///
/// This is a separate type from GuardId and FirstHopId to avoid confusion
/// about what kind of object we're identifying.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash, Ord, PartialOrd, AsRef)]
#[serde(transparent)]
pub(crate) struct GuardId(pub(crate) RelayIds);

impl GuardId {
    /// Return a new, manually constructed `GuardId`
    #[cfg(test)]
    pub(crate) fn new(ed25519: pk::ed25519::Ed25519Identity, rsa: pk::rsa::RsaIdentity) -> Self {
        Self(RelayIds::new(ed25519, rsa))
    }
    /// Extract a `GuardId` from a ChanTarget object.
    pub(crate) fn from_relay_ids<T>(target: &T) -> Self
    where
        T: tor_linkspec::HasRelayIds + ?Sized,
    {
        Self(RelayIds::from_relay_ids(target))
    }
}

/// Implementation type held inside of FirstHopId.
///
/// This exists as a separate type from FirstHopId because Rust requires that a pub enum's variants are all public.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum FirstHopIdInner {
    /// Identifies a guard.
    Guard(GuardId),
    /// Identifies a fallback.
    Fallback(FallbackId),
}

/// A unique cryptographic identifier for a selected guard or fallback
/// directory.
///
/// (This is implemented internally using all of the guard's known identities.)
///
/// TODO(nickm): we may want a fuzzier match for this type in the future in our
/// maps, if we ever learn about more identity types.  Right now we don't
/// recognize two `FirstHopId`s as "the same" if one has more IDs than the
/// other, even if all the IDs that they do have are the same.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct FirstHopId(pub(crate) FirstHopIdInner);

impl From<GuardId> for FirstHopId {
    fn from(id: GuardId) -> Self {
        Self(FirstHopIdInner::Guard(id))
    }
}
impl From<FallbackId> for FirstHopId {
    fn from(id: FallbackId) -> Self {
        Self(FirstHopIdInner::Fallback(id))
    }
}
impl AsRef<RelayIds> for FirstHopId {
    /// Return the inner `RelayIds` for this object.
    ///
    /// Only use this when it's okay to erase the type information about
    /// whether this identifies a guard or a fallback.
    fn as_ref(&self) -> &RelayIds {
        match &self.0 {
            FirstHopIdInner::Guard(id) => id.as_ref(),
            FirstHopIdInner::Fallback(id) => id.as_ref(),
        }
    }
}
impl tor_linkspec::HasRelayIds for FirstHopId {
    fn identity(
        &self,
        key_type: tor_linkspec::RelayIdType,
    ) -> Option<tor_linkspec::RelayIdRef<'_>> {
        self.as_ref().identity(key_type)
    }
}

impl FirstHopId {
    /// Return the relay in `netdir` that corresponds to this ID, if there
    /// is one.
    //
    // We have to define this function so it'll be public.
    pub fn get_relay<'a>(&self, netdir: &'a tor_netdir::NetDir) -> Option<tor_netdir::Relay<'a>> {
        netdir.by_ids(self)
    }
}
