//! Convenience implementation of a TimeBound object.

use std::ops::{Bound, RangeBounds};
use std::time;

/// A TimeBound object that is valid for a specified range of time.
///
/// The range is given as an argument, as in `t1..t2`.
///
///
/// ```
/// use std::time::{SystemTime, Duration};
/// use tor_checkable::{Timebound, TimeValidityError, timed::TimerangeBound};
///
/// let now = SystemTime::now();
/// let one_hour = Duration::new(3600, 0);
///
/// // This seven is only valid for another hour!
/// let seven = TimerangeBound::new(7_u32, ..now+one_hour);
///
/// assert_eq!(seven.check_valid_at(&now).unwrap(), 7);
///
/// // That consumed the previous seven. Try another one.
/// let seven = TimerangeBound::new(7_u32, ..now+one_hour);
/// assert_eq!(seven.check_valid_at(&(now+2*one_hour)),
///            Err(TimeValidityError::Expired(one_hour)));
///
/// ```
pub struct TimerangeBound<T> {
    /// The underlying object, which we only want to expose if it is
    /// currently timely.
    obj: T,
    /// If present, when the object first became valid.
    start: Option<time::SystemTime>,
    /// If present, when the object will no longer be valid.
    end: Option<time::SystemTime>,
}

/// Helper: convert a Bound to its underlying value, if any.
///
/// This helper discards information about whether the bound was
/// inclusive or exclusive.  However, since SystemTime has sub-second
/// precision, we really don't care about what happens when the
/// nanoseconds are equal to exactly 0.
fn unwrap_bound(b: Bound<&'_ time::SystemTime>) -> Option<time::SystemTime> {
    match b {
        Bound::Included(x) => Some(*x),
        Bound::Excluded(x) => Some(*x),
        _ => None,
    }
}

impl<T> TimerangeBound<T> {
    /// Construct a new TimerangeBound object from a given object and range.
    ///
    /// Note that we do not distinguish between inclusive and
    /// exclusive bounds: `x..y` and `x..=y` are treated the same
    /// here.
    pub fn new<U>(obj: T, range: U) -> Self
    where
        U: RangeBounds<time::SystemTime>,
    {
        let start = unwrap_bound(range.start_bound());
        let end = unwrap_bound(range.end_bound());
        Self { obj, start, end }
    }

    /// Adjust this time-range bound to tolerate an expiration time farther
    /// in the future.
    #[must_use]
    pub fn extend_tolerance(self, d: time::Duration) -> Self {
        let end = self.end.map(|t| t + d);
        Self { end, ..self }
    }
    /// Adjust this time-range bound to tolerate an initial validity
    /// time farther in the past.
    #[must_use]
    pub fn extend_pre_tolerance(self, d: time::Duration) -> Self {
        let start = self.start.map(|t| t - d);
        Self { start, ..self }
    }
}

impl<T> crate::Timebound<T> for TimerangeBound<T> {
    type Error = crate::TimeValidityError;

    fn is_valid_at(&self, t: &time::SystemTime) -> Result<(), Self::Error> {
        use crate::TimeValidityError;
        if let Some(start) = self.start {
            if let Ok(d) = start.duration_since(*t) {
                return Err(TimeValidityError::NotYetValid(d));
            }
        }

        if let Some(end) = self.end {
            if let Ok(d) = t.duration_since(end) {
                return Err(TimeValidityError::Expired(d));
            }
        }

        Ok(())
    }

    fn dangerously_assume_timely(self) -> T {
        self.obj
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{TimeValidityError, Timebound};
    use std::time::{Duration, SystemTime};

    #[test]
    fn test_bounds() {
        let one_day = Duration::new(86400, 0);
        let mixminion_v0_0_1 = SystemTime::UNIX_EPOCH + 12059 * one_day; //2003-01-07
        let tor_v0_0_2pre13 = SystemTime::UNIX_EPOCH + 12344 * one_day; //2003-10-19
        let cussed_nougat = SystemTime::UNIX_EPOCH + 14093 * one_day; //2008-08-02
        let tor_v0_4_4_5 = SystemTime::UNIX_EPOCH + 18520 * one_day; //2020-09-15
        let today = SystemTime::UNIX_EPOCH + 18527 * one_day; //2020-09-22

        let tr = TimerangeBound::new((), ..tor_v0_4_4_5);
        assert_eq!(tr.start, None);
        assert_eq!(tr.end, Some(tor_v0_4_4_5));
        assert!(tr.is_valid_at(&mixminion_v0_0_1).is_ok());
        assert!(tr.is_valid_at(&tor_v0_0_2pre13).is_ok());
        assert_eq!(
            tr.is_valid_at(&today),
            Err(TimeValidityError::Expired(7 * one_day))
        );

        let tr = TimerangeBound::new((), tor_v0_0_2pre13..=tor_v0_4_4_5);
        assert_eq!(tr.start, Some(tor_v0_0_2pre13));
        assert_eq!(tr.end, Some(tor_v0_4_4_5));
        assert_eq!(
            tr.is_valid_at(&mixminion_v0_0_1),
            Err(TimeValidityError::NotYetValid(285 * one_day))
        );
        assert!(tr.is_valid_at(&cussed_nougat).is_ok());
        assert_eq!(
            tr.is_valid_at(&today),
            Err(TimeValidityError::Expired(7 * one_day))
        );

        let tr = tr
            .extend_pre_tolerance(5 * one_day)
            .extend_tolerance(2 * one_day);
        assert_eq!(tr.start, Some(tor_v0_0_2pre13 - 5 * one_day));
        assert_eq!(tr.end, Some(tor_v0_4_4_5 + 2 * one_day));

        let tr = TimerangeBound::new((), tor_v0_4_4_5..);
        assert_eq!(tr.start, Some(tor_v0_4_4_5));
        assert_eq!(tr.end, None);
        assert_eq!(
            tr.is_valid_at(&cussed_nougat),
            Err(TimeValidityError::NotYetValid(4427 * one_day))
        );
        assert!(tr.is_valid_at(&today).is_ok());
    }

    #[test]
    fn test_checking() {
        let one_day = Duration::new(86400, 0);
        let de = SystemTime::UNIX_EPOCH + one_day * 7580;
        let cz_sk = SystemTime::UNIX_EPOCH + one_day * 8401;
        let eu = SystemTime::UNIX_EPOCH + one_day * 8705;
        let za = SystemTime::UNIX_EPOCH + one_day * 8882;

        let tr = TimerangeBound::new("Hello world", cz_sk..eu);
        assert!(tr.check_valid_at(&za).is_err());

        let tr = TimerangeBound::new("Hello world", cz_sk..za);
        assert_eq!(tr.check_valid_at(&eu), Ok("Hello world"));

        let tr = TimerangeBound::new("hello world", de..);
        assert_eq!(tr.check_valid_now(), Ok("hello world"));

        let tr = TimerangeBound::new("hello world", ..za);
        assert!(tr.check_valid_now().is_err());
    }
}
