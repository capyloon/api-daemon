//! Declare the [`FallbackState`] type, which is used to store a set of FallbackDir.

use rand::seq::IteratorRandom;
use std::time::Instant;

use super::{FallbackDir, Status};
use crate::{ids::FallbackId, PickGuardError};
use serde::Deserialize;

/// A list of fallback directories.
///
/// Fallback directories (represented by [`FallbackDir`]) are used by Tor
/// clients when they don't already have enough other directory information to
/// contact the network.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct FallbackList {
    /// The underlying fallbacks in this set.
    fallbacks: Vec<FallbackDir>,
}

impl<T: IntoIterator<Item = FallbackDir>> From<T> for FallbackList {
    fn from(fallbacks: T) -> Self {
        FallbackList {
            fallbacks: fallbacks.into_iter().collect(),
        }
    }
}

impl FallbackList {
    /// Return the number of fallbacks in this list.
    pub fn len(&self) -> usize {
        self.fallbacks.len()
    }
    /// Return true if there are no fallbacks in this list.
    pub fn is_empty(&self) -> bool {
        self.fallbacks.is_empty()
    }
    /// Return a random member of this list.
    pub fn choose<R: rand::Rng>(&self, rng: &mut R) -> Result<&FallbackDir, PickGuardError> {
        // TODO: Return NoCandidatesAvailable when the fallback list is empty.
        self.fallbacks
            .iter()
            .choose(rng)
            .ok_or(PickGuardError::AllFallbacksDown { retry_at: None })
    }
}

/// A set of fallback directories, in usable form.
#[derive(Debug, Clone)]
pub(crate) struct FallbackState {
    /// The list of fallbacks in the set.
    ///
    /// We require that these are sorted and unique by (ED,RSA) keys.
    fallbacks: Vec<Entry>,
}

/// Wrapper type for FallbackDir converted into crate::Guard, and Status.
///
/// Defines a sort order to ensure that we can look up fallback directories
/// by binary search on keys.
#[derive(Debug, Clone)]
pub(super) struct Entry {
    /// The inner fallback directory.
    ///
    /// (TODO: We represent this as a `FirstHop`, which could technically hold a
    /// guard as well.  Ought to fix that.)
    pub(super) fallback: crate::FirstHop,
    /// The status for the fallback directory.
    pub(super) status: Status,
}

impl From<FallbackDir> for Entry {
    fn from(fallback: FallbackDir) -> Self {
        let fallback = fallback.as_guard();
        let status = Status::default();
        Entry { fallback, status }
    }
}

impl Entry {
    /// Return the identity for this fallback entry.
    fn id(&self) -> &FallbackId {
        use crate::ids::FirstHopIdInner::*;
        match &self.fallback.id().0 {
            Fallback(id) => id,
            _ => panic!("Somehow we constructed a fallback object with a non-fallback id!"),
        }
    }
}

impl From<FallbackList> for FallbackState {
    fn from(list: FallbackList) -> Self {
        let mut fallbacks: Vec<Entry> = list.fallbacks.into_iter().map(|fb| fb.into()).collect();
        fallbacks.sort_by(|x, y| x.id().cmp(y.id()));
        fallbacks.dedup_by(|x, y| x.id() == y.id());
        FallbackState { fallbacks }
    }
}

impl FallbackState {
    /// Return a random member of this FallbackSet that's usable at `now`.
    pub(crate) fn choose<R: rand::Rng>(
        &self,
        rng: &mut R,
        now: Instant,
    ) -> Result<&crate::FirstHop, PickGuardError> {
        if self.fallbacks.is_empty() {
            return Err(PickGuardError::NoCandidatesAvailable);
        }

        self.fallbacks
            .iter()
            .filter(|ent| ent.status.usable_at(now))
            .choose(rng)
            .map(|ent| &ent.fallback)
            .ok_or_else(|| PickGuardError::AllFallbacksDown {
                retry_at: self.next_retry(),
            })
    }

    /// Return the next time at which any member of this set will become ready.
    ///
    /// Returns None if no elements are failing.
    fn next_retry(&self) -> Option<Instant> {
        self.fallbacks
            .iter()
            .filter_map(|ent| ent.status.next_retriable())
            .min()
    }

    /// Return a reference to the entry whose identity is `id`, if there is one.
    fn get(&self, id: &FallbackId) -> Option<&Entry> {
        match self.fallbacks.binary_search_by(|e| e.id().cmp(id)) {
            Ok(idx) => Some(&self.fallbacks[idx]),
            Err(_) => None,
        }
    }

    /// Return a mutable reference to the entry whose identity is `id`, if there is one.
    fn get_mut(&mut self, id: &FallbackId) -> Option<&mut Entry> {
        match self.fallbacks.binary_search_by(|e| e.id().cmp(id)) {
            Ok(idx) => Some(&mut self.fallbacks[idx]),
            Err(_) => None,
        }
    }

    /// Return true if this set contains some entry with the given `id`.
    pub(crate) fn contains(&self, id: &FallbackId) -> bool {
        self.get(id).is_some()
    }

    /// Record that a success has occurred for the fallback with the given
    /// identity.
    ///
    /// Be aware that for fallbacks, we only count a successful directory
    /// operation as a success: a circuit success is not enough.
    pub(crate) fn note_success(&mut self, id: &FallbackId) {
        if let Some(entry) = self.get_mut(id) {
            entry.status.note_success();
        }
    }

    /// Record that a failure has occurred for the fallback with the given
    /// identity.
    pub(crate) fn note_failure(&mut self, id: &FallbackId, now: Instant) {
        if let Some(entry) = self.get_mut(id) {
            entry.status.note_failure(now);
        }
    }

    /// Consume `other` and copy all of its fallback status entries into the corresponding entries for `self`.
    pub(crate) fn take_status_from(&mut self, other: FallbackState) {
        use itertools::EitherOrBoth::Both;

        itertools::merge_join_by(
            self.fallbacks.iter_mut(),
            other.fallbacks.into_iter(),
            |a, b| a.fallback.id().cmp(b.fallback.id()),
        )
        .for_each(|entry| {
            if let Both(entry, other) = entry {
                debug_assert_eq!(entry.fallback.id(), other.fallback.id());
                entry.status = other.status;
            }
        });
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::FirstHopId;

    /// Construct a `FallbackDir` with random identity keys and addresses.
    ///
    /// Since there are 416 bits of random id here, the risk of collision is
    /// negligible.
    fn rand_fb() -> FallbackDir {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let ed: [u8; 32] = rng.gen();
        let rsa: [u8; 20] = rng.gen();
        let ip: u32 = rng.gen();
        FallbackDir::builder()
            .ed_identity(ed.into())
            .rsa_identity(rsa.into())
            .orport(std::net::SocketAddrV4::new(ip.into(), 9090).into())
            .build()
            .unwrap()
    }

    #[test]
    fn construct_fallback_set() {
        use rand::seq::SliceRandom;

        // fabricate some fallbacks.
        let fbs = vec![rand_fb(), rand_fb(), rand_fb(), rand_fb()];
        let fb_other = rand_fb();
        let id_other = FallbackId::from_chan_target(&fb_other);

        // basic case: construct a set
        let list: FallbackList = fbs.clone().into();
        assert!(!list.is_empty());
        assert_eq!(list.len(), 4);
        let mut set: FallbackState = list.clone().into();

        // inspect the generated set
        assert_eq!(set.fallbacks.len(), 4);
        assert!(set.fallbacks[0].id() < set.fallbacks[1].id());
        assert!(set.fallbacks[1].id() < set.fallbacks[2].id());
        assert!(set.fallbacks[2].id() < set.fallbacks[3].id());

        // use the constructed set a little.
        for fb in fbs.iter() {
            let id = FallbackId::from_chan_target(fb);
            assert_eq!(set.get_mut(&id).unwrap().id(), &id);
        }
        assert!(set.get_mut(&id_other).is_none());

        // Now try an input set with duplicates.
        let mut redundant_fbs = fbs.clone();
        redundant_fbs.extend(fbs.clone());
        redundant_fbs.extend(fbs[0..2].iter().map(Clone::clone));
        redundant_fbs[..].shuffle(&mut rand::thread_rng());
        let list2 = redundant_fbs.into();
        assert_ne!(&list, &list2);
        let set2: FallbackState = list2.into();

        // It should have the same elements, in the same order.
        assert_eq!(set.fallbacks.len(), set2.fallbacks.len());
        assert!(set
            .fallbacks
            .iter()
            .zip(set2.fallbacks.iter())
            .all(|(ent1, ent2)| ent1.id() == ent2.id()));
    }

    #[test]
    fn set_choose() {
        let fbs = vec![rand_fb(), rand_fb(), rand_fb(), rand_fb()];
        let list: FallbackList = fbs.into();
        let mut set: FallbackState = list.into();

        let mut counts = [0_usize; 4];
        let mut rng = rand::thread_rng();
        let now = Instant::now();

        fn lookup_idx(set: &FallbackState, id: &FirstHopId) -> Option<usize> {
            if let FirstHopId(crate::ids::FirstHopIdInner::Fallback(id)) = id {
                set.fallbacks.binary_search_by(|ent| ent.id().cmp(id)).ok()
            } else {
                None
            }
        }
        // Basic case: everybody is up.
        for _ in 0..100 {
            let fb = set.choose(&mut rng, now).unwrap();
            let idx = lookup_idx(&set, fb.id()).unwrap();
            counts[idx] += 1;
        }
        assert!(counts.iter().all(|v| *v > 0));

        // Mark somebody down and make sure they don't get chosen.
        let ids: Vec<_> = set.fallbacks.iter().map(|fb| fb.id().clone()).collect();
        set.note_failure(&ids[2], now);
        counts = [0; 4];
        for _ in 0..100 {
            let fb = set.choose(&mut rng, now).unwrap();
            let idx = lookup_idx(&set, fb.id()).unwrap();
            counts[idx] += 1;
        }
        assert_eq!(counts.iter().filter(|v| **v > 0).count(), 3);
        assert_eq!(counts[2], 0);

        // Mark everybody down; make sure we get the right error.
        for id in ids.iter() {
            set.note_failure(id, now);
        }
        assert!(matches!(
            set.choose(&mut rng, now),
            Err(PickGuardError::AllFallbacksDown { .. })
        ));

        // Construct an empty set; make sure we get the right error.
        let empty_set = FallbackState::from(FallbackList::from(vec![]));
        assert!(matches!(
            empty_set.choose(&mut rng, now),
            Err(PickGuardError::NoCandidatesAvailable)
        ));

        // TODO: test restrictions and filters once they're implemented.
    }

    #[test]
    fn test_status() {
        let fbs = vec![rand_fb(), rand_fb(), rand_fb(), rand_fb()];
        let list: FallbackList = fbs.clone().into();
        let mut set: FallbackState = list.into();
        let ids: Vec<_> = set.fallbacks.iter().map(|fb| fb.id().clone()).collect();

        let now = Instant::now();

        // There's no "next retry time" when everybody's up.
        assert!(set.next_retry().is_none());

        // Mark somebody down; try accessors.
        set.note_failure(&ids[3], now);
        assert!(set.fallbacks[3].status.next_retriable().unwrap() > now);
        assert!(!set.fallbacks[3].status.usable_at(now));
        assert_eq!(set.next_retry(), set.fallbacks[3].status.next_retriable());

        // Mark somebody else down; try accessors.
        set.note_failure(&ids[0], now);
        assert!(set.fallbacks[0].status.next_retriable().unwrap() > now);
        assert!(!set.fallbacks[0].status.usable_at(now));
        assert_eq!(
            set.next_retry().unwrap(),
            std::cmp::min(
                set.fallbacks[0].status.next_retriable().unwrap(),
                set.fallbacks[3].status.next_retriable().unwrap()
            )
        );

        // Mark somebody as running; try accessors.
        set.note_success(&ids[0]);
        assert!(set.fallbacks[0].status.next_retriable().is_none());
        assert!(set.fallbacks[0].status.usable_at(now));

        assert!(set.get_mut(&ids[0]).unwrap().status.usable_at(now));

        for id in ids.iter() {
            dbg!(id, set.get_mut(id).map(|e| e.id()));
        }

        // Make a new set with slightly different members; make sure that we can copy stuff successfully.
        let mut fbs2: Vec<_> = fbs
            .into_iter()
            // (Remove the fallback with id==ids[2])
            .filter(|fb| FallbackId::from_chan_target(fb) != ids[2])
            .collect();
        // add 2 new ones.
        let fbs_new = [rand_fb(), rand_fb(), rand_fb()];
        fbs2.extend(fbs_new.clone());

        let mut set2 = FallbackState::from(FallbackList::from(fbs2.clone()));
        set2.take_status_from(set); // consumes set.
        assert_eq!(set2.fallbacks.len(), 6); // Started with 4, added 3, removed 1.

        // Make sure that the status entries  are correctly copied.
        assert!(set2.get_mut(&ids[0]).unwrap().status.usable_at(now));
        assert!(set2.get_mut(&ids[1]).unwrap().status.usable_at(now));
        assert!(set2.get_mut(&ids[2]).is_none());
        assert!(!set2.get_mut(&ids[3]).unwrap().status.usable_at(now));

        // Make sure that the new fbs are there.
        for new_fb in fbs_new {
            assert!(set2
                .get_mut(&FallbackId::from_chan_target(&new_fb))
                .unwrap()
                .status
                .usable_at(now));
        }
    }
}
