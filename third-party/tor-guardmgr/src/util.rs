//! Helper functionality used by the rest of `tor-guardmgr`.

use rand::Rng;
use std::time::{Duration, SystemTime};

/// Return a random time within the range `when-max ..= when`.
///
/// Uses a uniform distribution; saturates at UNIX_EPOCH.  Rounds down to the
/// nearest 10 seconds.
///
/// This kind of date randomization is used in our persistent state in
/// an attempt to make some kinds of traffic analysis attacks a bit
/// harder for an attacker who can read our state after the fact.
pub(crate) fn randomize_time<R: Rng>(rng: &mut R, when: SystemTime, max: Duration) -> SystemTime {
    let offset = rng.gen_range(Duration::ZERO..max);
    let random = when
        .checked_sub(offset)
        .unwrap_or(SystemTime::UNIX_EPOCH)
        .max(SystemTime::UNIX_EPOCH);
    // Round to the nearest 10-second increment.
    round_time(random, 10)
}

/// Round `when` to a multiple of `d` seconds relative to epoch.
///
/// Rounds towards the epoch.
///
/// There's no reason to actually do this, since the times we use it
/// on are randomized, but rounding times in this way avoids giving a
/// false impression that we're storing hyper-accurate numbers.
///
/// # Panics
///
/// Panics if d == 0.
fn round_time(when: SystemTime, d: u32) -> SystemTime {
    let (early, elapsed) = if when < SystemTime::UNIX_EPOCH {
        (
            true,
            SystemTime::UNIX_EPOCH
                .duration_since(when)
                .expect("logic_error"),
        )
    } else {
        (
            false,
            when.duration_since(SystemTime::UNIX_EPOCH)
                .expect("logic error"),
        )
    };

    let secs_elapsed = elapsed.as_secs();
    let secs_rounded = secs_elapsed - (secs_elapsed % u64::from(d));
    let dur_rounded = Duration::from_secs(secs_rounded);

    if early {
        SystemTime::UNIX_EPOCH - dur_rounded
    } else {
        SystemTime::UNIX_EPOCH + dur_rounded
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn test_randomize_time() {
        let now = SystemTime::now();
        let one_hour = Duration::from_secs(3600);
        let ten_sec = Duration::from_secs(10);
        let mut rng = rand::thread_rng();

        for _ in 0..1000 {
            let t = randomize_time(&mut rng, now, one_hour);
            assert!(t >= now - one_hour - ten_sec);
            assert!(t <= now);
        }

        let close_to_epoch = SystemTime::UNIX_EPOCH + one_hour / 2;
        for _ in 0..1000 {
            let t = randomize_time(&mut rng, close_to_epoch, one_hour);
            assert!(t >= SystemTime::UNIX_EPOCH);
            assert!(t <= close_to_epoch);
            let d = t.duration_since(SystemTime::UNIX_EPOCH).unwrap();
            assert_eq!(d.subsec_nanos(), 0);
            assert_eq!(d.as_secs() % 10, 0);
        }
    }
}
