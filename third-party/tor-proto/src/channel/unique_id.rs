//! Helper for unique identifiers for channels.

use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Counter for allocating unique-ish identifiers for channels.
static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

/// Unique identifier for a channel.
///
/// These identifiers are unique per process.  On 32-bit architectures
/// it's possible to exhaust them if you do nothing but create channels
/// for a very long time; if you do, we detect that and exit with an
/// assertion failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UniqId(usize);

impl UniqId {
    /// Construct a new UniqId.
    pub(crate) fn new() -> Self {
        // Relaxed ordering is fine; we don't care about how this
        // is instantiated with respect to other channels.
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        assert!(id != std::usize::MAX, "Exhausted the channel ID namespace");
        UniqId(id)
    }
}

impl Display for UniqId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Chan {}", self.0)
    }
}

/// Counter for allocating circuit unique ids.
///
/// We don't use circuit IDs here, because they can be huge, and they're
/// random, and can get reused.
#[derive(Debug)]
pub(crate) struct CircUniqIdContext {
    /// Next value to be handed out for this channel's circuits.
    next_circ_id: usize,
}

impl CircUniqIdContext {
    /// Create a new CircUniqIdContext
    pub(super) fn new() -> Self {
        CircUniqIdContext { next_circ_id: 0 }
    }
    /// Construct a new, unique-ish circuit UniqId
    pub(super) fn next(&mut self, unique_id: UniqId) -> crate::circuit::UniqId {
        let circ_unique_id = self.next_circ_id;
        self.next_circ_id += 1;
        assert!(
            self.next_circ_id != 0,
            "Exhausted the unique circuit ID namespace on a channel"
        );
        crate::circuit::UniqId::new(unique_id.0, circ_unique_id)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn chan_unique_id() {
        let ids: Vec<UniqId> = (0..10).map(|_| UniqId::new()).collect();

        // Make sure we got distinct numbers
        let mut all_nums: Vec<_> = ids.iter().map(|x| x.0).collect();
        all_nums.sort_unstable();
        all_nums.dedup();
        assert_eq!(all_nums.len(), ids.len());

        assert_eq!(format!("{}", ids[3]), format!("Chan {}", ids[3].0));
    }

    #[test]
    fn chan_circid() {
        let chan_id99 = UniqId(99);
        let mut ctx = CircUniqIdContext::new();

        let _id0 = ctx.next(chan_id99);
        let _id1 = ctx.next(chan_id99);
        let id2 = ctx.next(chan_id99);
        assert_eq!(format!("{}", id2), "Circ 99.2");
    }
}
