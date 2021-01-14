#![deny(missing_docs, warnings)]
#![cfg_attr(test, feature(test))]

//! A mutex which can only be locked once, but which provides
//! very fast concurrent reads after the first lock is over.
//!
//! ## Example
//!
//! ```
//! # use oncemutex::OnceMutex;
//!
//! let mutex = OnceMutex::new(8);
//!
//! // One-time lock
//! *mutex.lock().unwrap() = 9;
//!
//! // Cheap lock-free access.
//! assert_eq!(*mutex, 9);
//! ```
//!

#[cfg(test)]
extern crate test;

use std::sync::{Mutex, MutexGuard};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::mem;

const UNUSED: usize = 0;
const LOCKED: usize = 1;
const FREE: usize = 2;

/// A mutex which can only be locked once, but which provides
/// very fast, lock-free, concurrent reads after the first
/// lock is over.
pub struct OnceMutex<T> {
    lock: Mutex<()>,
    state: AtomicUsize,
    data: UnsafeCell<T>
}

unsafe impl<T: Send> Send for OnceMutex<T> {}
unsafe impl<T: Sync> Sync for OnceMutex<T> {}

impl<T: Send + Sync> OnceMutex<T> {
    /// Create a new OnceMutex.
    pub fn new(x: T) -> OnceMutex<T> {
        OnceMutex {
            lock: Mutex::new(()),
            state: AtomicUsize::new(UNUSED),
            data: UnsafeCell::new(x)
        }
    }

    /// Attempt to lock the OnceMutex.
    ///
    /// This will not block, but will return None if the OnceMutex
    /// has already been locked or is currently locked by another thread.
    pub fn lock(&self) -> Option<OnceMutexGuard<T>> {
        match self.state.compare_and_swap(UNUSED, LOCKED, SeqCst) {
            // self.state is now LOCKED.
            UNUSED => {
                // Locks self.lock
                Some(OnceMutexGuard::new(self))
            },

            // Other thread got here first or already locked.
            // Either way, no lock.
            _ => None
        }
    }

    /// Block the current task until the first lock is over.
    ///
    /// Does nothing if there is no lock.
    pub fn wait(&self) {
        // Don't take out a lock if we aren't locked.
        if self.locked() { let _ = self.lock.lock(); }
    }

    /// Extract the data from a OnceMutex.
    pub fn into_inner(self) -> T {
        unsafe { self.data.into_inner() }
    }

    /// Is this OnceMutex currently locked?
    pub fn locked(&self) -> bool {
        self.state.load(SeqCst) == LOCKED
    }
}

impl<T: Send + Sync> Deref for OnceMutex<T> {
    type Target = T;

    /// Get a reference to the value inside the OnceMutex.
    ///
    /// This can block if the OnceMutex is in its lock, but is
    /// very fast otherwise.
    fn deref(&self) -> &T {
        if LOCKED == self.state.compare_and_swap(UNUSED, FREE, SeqCst) {
            // The OnceMutexGuard has not released yet.
            self.wait();
        }

        debug_assert_eq!(self.state.load(SeqCst), FREE);

        // We are FREE, so go!
        unsafe { mem::transmute(self.data.get()) }
    }
}

// Safe, because we have &mut self, which means no OnceMutexGuard's exist.
impl<T: Send + Sync> DerefMut for OnceMutex<T> {
    fn deref_mut(&mut self) -> &mut T {
        // Should be impossible.
        debug_assert!(self.state.load(SeqCst) != LOCKED);

        unsafe { mem::transmute(self.data.get()) }
    }
}

/// A guard providing a one-time lock on a OnceMutex.
pub struct OnceMutexGuard<'a, T: 'a> {
    parent: &'a OnceMutex<T>,
    // Only used for its existence, so triggers dead_code warnings.
    _lock: MutexGuard<'a, ()>
}

impl<'a, T> OnceMutexGuard<'a, T> {
    fn new(mutex: &'a OnceMutex<T>) -> OnceMutexGuard<'a, T> {
        OnceMutexGuard {
            parent: mutex,
            _lock: mutex.lock.lock().unwrap()
        }
    }
}

impl<'a, T: Send + Sync> DerefMut for OnceMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { mem::transmute(self.parent.data.get()) }
    }
}

impl<'a, T: Send + Sync> Deref for OnceMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { mem::transmute(self.parent.data.get()) }
    }
}

impl<'a, T> Drop for OnceMutexGuard<'a, T> {
    fn drop(&mut self) {
        self.parent.state.store(FREE, SeqCst);
    }
}

fn _assert_send_sync() {
    fn _assert_send<T: Send>() {}
    fn _assert_sync<T: Sync>() {}

    _assert_send::<OnceMutex<Vec<u8>>>();
    _assert_sync::<OnceMutex<Vec<u8>>>();
}

#[cfg(test)]
mod tests {
    use super::{OnceMutex, FREE, UNUSED, LOCKED};
    use std::sync::atomic::Ordering::SeqCst;
    use std::sync::Mutex;
    use {test};

    #[test]
    fn test_once_mutex_locks_only_once() {
        let mutex = OnceMutex::new("hello");
        assert!(mutex.lock().is_some());
        assert!(mutex.lock().is_none());
        assert!(mutex.lock().is_none());
    }

    #[test]
    fn test_once_mutex_states() {
        let mutex = OnceMutex::new("hello");
        assert_eq!(mutex.state.load(SeqCst), UNUSED);

        let lock = mutex.lock();
        assert_eq!(mutex.state.load(SeqCst), LOCKED);

        drop(lock);
        assert_eq!(mutex.state.load(SeqCst), FREE);
    }

    #[test]
    fn test_once_mutex_deref() {
        let mutex = OnceMutex::new("hello");
        *mutex;
        assert_eq!(mutex.state.load(SeqCst), FREE);
    }

    #[bench]
    fn bench_once_mutex_locking(bencher: &mut test::Bencher) {
        let mutex = OnceMutex::new(5);
        bencher.iter(|| {
            ::test::black_box(mutex.lock());
            mutex.state.store(UNUSED, SeqCst);
        });
    }

    #[bench]
    fn bench_once_mutex_access(bencher: &mut test::Bencher) {
        let mutex = OnceMutex::new(5);
        bencher.iter(|| ::test::black_box(*mutex));
    }

    #[bench]
    fn bench_mutex_locking(bencher: &mut test::Bencher) {
        let mutex = Mutex::new(5);
        bencher.iter(|| ::test::black_box(mutex.lock()));
    }
}

