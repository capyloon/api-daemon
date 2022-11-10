/// Scorer based on the frecency algorithm
/// See https://developer.mozilla.org/en-US/docs/Mozilla/Tech/Places/Frecency_algorithm
use chrono::{DateTime, Utc};
use libsqlite3_sys::{
    sqlite3_context, sqlite3_result_int, sqlite3_value, sqlite3_value_blob, sqlite3_value_bytes,
};
use speedy::{Readable, Writable};
use std::os::raw::c_int;

static MAX_VISIT_ENTRIES: usize = 10;

#[derive(Debug, Clone, Readable, Writable)]
pub enum VisitPriority {
    Normal,
    High,
    VeryHigh,
}

impl VisitPriority {
    // Percentage bonus based on the visit priority.
    pub fn bonus(&self) -> u32 {
        match &self {
            Self::Normal => 100,
            Self::High => 150,
            Self::VeryHigh => 200,
        }
    }
}

#[derive(Debug, Clone, Readable, Writable)]
pub struct VisitEntry {
    pub timestamp: i64, // Time since EPOCH in nano seconds.
    pub priority: VisitPriority,
}

impl VisitEntry {
    pub fn new(when: &DateTime<Utc>, priority: VisitPriority) -> Self {
        Self {
            timestamp: (*when).naive_utc().timestamp_nanos(),
            priority,
        }
    }

    pub fn now(priority: VisitPriority) -> Self {
        Self {
            timestamp: Utc::now().naive_utc().timestamp_nanos(),
            priority,
        }
    }
}

#[derive(Clone, Debug, Readable, Writable)]
pub struct Scorer {
    all_time_visits: u32, // The total number of visits, which can be greater than the entries we keep.
    #[speedy(length_type = u8)] // u8 is enough since MAX_VISIT_ENTRIES < 255
    entries: Vec<VisitEntry>,
}

fn weight_for(when: i64) -> u32 {
    use chrono::TimeZone;

    let days = (Utc::now() - Utc.timestamp_nanos(when)).num_days();
    if days <= 4 {
        100
    } else if days <= 14 {
        70
    } else if days <= 31 {
        50
    } else if days <= 90 {
        30
    } else {
        10
    }
}

impl Default for Scorer {
    fn default() -> Self {
        Self {
            all_time_visits: 0,
            entries: Vec::with_capacity(MAX_VISIT_ENTRIES),
        }
    }
}

impl Scorer {
    pub fn add(&mut self, entry: &VisitEntry) {
        // Remove the oldest entry to make room for the new one.
        if self.entries.len() == MAX_VISIT_ENTRIES {
            let _ = self.entries.remove(0);
        }

        self.entries.push(entry.clone());
        self.all_time_visits += 1;
    }

    // Used for bench
    // pub fn frecency_float(&self) -> u32 {
    //     if self.entries.is_empty() {
    //         return 0;
    //     }

    //     // For each sampled visit, the score is (bonus / 100.0) * weight
    //     // The final score for each item is ceiling(total visit count * sum of points for sampled visits / number of sampled visits)

    //     let sum = (&self.entries)
    //         .iter()
    //         .map(|item| (item.priority.bonus() * weight_for(item.timestamp)) as f32 / 100.0)
    //         .sum::<f32>();

    //     self.all_time_visits * sum.round() as u32 / self.entries.len() as u32
    // }

    pub fn frecency(&self) -> u32 {
        if self.entries.is_empty() {
            return 0;
        }

        // For each sampled visit, the score is (bonus / 100.0) * weight
        // The final score for each item is ceiling(total visit count * sum of points for sampled visits / number of sampled visits)

        let sum = self
            .entries
            .iter()
            .map(|item| (item.priority.bonus() * weight_for(item.timestamp)))
            .sum::<u32>();

        self.all_time_visits * sum / (100 * self.entries.len() as u32)
    }

    #[cfg(test)]
    pub fn max() -> u32 {
        let mut score = Scorer::default();
        let now = Utc::now();
        for _i in 0..MAX_VISIT_ENTRIES {
            score.add(&VisitEntry::new(&now, VisitPriority::VeryHigh));
        }
        score.frecency()
    }

    pub fn as_binary(&self) -> Vec<u8> {
        self.write_to_vec().unwrap()
    }

    pub fn from_binary(input: &[u8]) -> Self {
        Self::read_from_buffer(input).expect("Failed to deserialize scorer")
    }
}

impl PartialEq for Scorer {
    fn eq(&self, other: &Scorer) -> bool {
        self.frecency() == other.frecency()
    }
}

/// # Safety
///
/// SQlite function to return an up to date value of the frecency.
pub unsafe extern "C" fn sqlite_frecency(
    ctx: *mut sqlite3_context,
    argc: c_int,
    argv: *mut *mut sqlite3_value,
) {
    // 0. Check argument count.
    if argc != 1 {
        sqlite3_result_int(ctx, 0);
        return;
    }

    // 1. Get the blob from the first argument.
    let args = std::slice::from_raw_parts(argv, argc as _);
    let blob_arg = args[0];

    let len = sqlite3_value_bytes(blob_arg) as usize;

    if len == 0 {
        // empty blobs are NULL so just return 0.
        sqlite3_result_int(ctx, 0);
        return;
    }
    let ptr = sqlite3_value_blob(blob_arg) as *const u8;
    debug_assert!(!ptr.is_null());
    let array = std::slice::from_raw_parts(ptr, len);

    // 2. Get a Scorer object and return the frecency.
    let scorer = Scorer::from_binary(array);
    sqlite3_result_int(ctx, scorer.frecency() as _);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frecency_alg() {
        use chrono::Duration;

        assert_eq!(Scorer::max(), 2000);

        // Add 2 visits of normal priority with a 10 day interval.
        let mut score = Scorer::default();
        assert_eq!(score.frecency(), 0);
        // assert_eq!(score.frecency(), score.frecency_float());

        let now = Utc::now();
        score.add(&VisitEntry::new(&now, VisitPriority::Normal));
        assert_eq!(score.frecency(), 100);
        // assert_eq!(score.frecency(), score.frecency_float());

        score.add(&VisitEntry::new(
            &(now - Duration::days(10)),
            VisitPriority::Normal,
        ));
        assert_eq!(score.frecency(), 170);
        // assert_eq!(score.frecency(), score.frecency_float());

        // Add 2 visits with a 10 day interval, one with high priority.
        let mut score = Scorer::default();

        let now = Utc::now();
        score.add(&VisitEntry::new(&now, VisitPriority::Normal));
        assert_eq!(score.frecency(), 100);
        // assert_eq!(score.frecency(), score.frecency_float());

        score.add(&VisitEntry::new(
            &(now - Duration::days(10)),
            VisitPriority::High,
        ));
        assert_eq!(score.frecency(), 205);
        // assert_eq!(score.frecency(), score.frecency_float());
    }
}

// use test::Bencher;

// #[bench]
// fn bench_frecency_int(b: &mut Bencher) {
//     use chrono::Duration;

//     let mut score = Scorer::default();
//     let now = Utc::now();
//     score.add(&VisitEntry::new(&now, VisitPriority::Normal));
//     score.add(&VisitEntry::new(
//         &(now - Duration::days(10)),
//         VisitPriority::Normal,
//     ));
//     score.add(&VisitEntry::new(
//         &(now - Duration::days(20)),
//         VisitPriority::High,
//     ));
//     let bytes = score.as_bincode();

//     b.iter(|| {
//         let score = Scorer::from_bincode(&bytes);
//         let _frec = score.frecency();
//     });
// }

// #[bench]
// fn bench_frecency_float(b: &mut Bencher) {
//     use chrono::Duration;

//     let mut score = Scorer::default();
//     let now = Utc::now();
//     score.add(&VisitEntry::new(&now, VisitPriority::Normal));
//     for i in 1..10 {
//         score.add(&VisitEntry::new(
//             &(now - Duration::days(i)),
//             VisitPriority::Normal,
//         ));
//     }
//     score.add(&VisitEntry::new(
//         &(now - Duration::days(20)),
//         VisitPriority::High,
//     ));
//     let bytes = score.as_bincode();

//     b.iter(|| {
//         let score = Scorer::from_bincode(&bytes);
//         let _frec = score.frecency_float();
//     });
// }
