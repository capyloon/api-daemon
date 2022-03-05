//! A double-ended queue|ringbuffer with an upper bound on its length.
//!
//! The primary item of interest in this crate is the [`BoundedVecDeque`] type, which is the
//! double-ended q.r. with an etc. that was mentioned above.
//!
//! This crate requires Rust 1.28.0 or later.
//!
//! Much of the documentation of this crate was copied (with some modifications) from [the
//! `VecDeque` documentation][`VecDeque`] and other documentation of the Rust standard library.
//!
//! # Features
//!
//! The following crate features exist:
//!
//! - `fused` (enabled by default): Implements [`FusedIterator`] for the various iterator types.
//! - `resize_with` (requires Rust 1.33): Adds [`resize_with()`].
//!
//! [`VecDeque`]: https://doc.rust-lang.org/std/collections/struct.VecDeque.html
//! [`BoundedVecDeque`]: struct.BoundedVecDeque.html
//! [`FusedIterator`]: https://doc.rust-lang.org/std/iter/trait.FusedIterator.html
//! [`resize_with()`]: struct.BoundedVecDeque.html#method.resize_with

#![forbid(unsafe_code, bare_trait_objects)]
#![warn(missing_docs)]

use ::std::collections::VecDeque;
use ::std::hash::{Hash, Hasher};
use ::std::ops::{Deref, Index, IndexMut, RangeBounds};

mod iter;
mod test;

pub use ::iter::{Iter, IterMut, IntoIter, Drain, Append};

/// A double-ended queue|ringbuffer with an upper bound on its length.
///
/// The “default” usage of this type as a queue is to use [`push_back()`] to add to the queue, and
/// [`pop_front()`] to remove from the queue. [`extend()`], [`append()`], and [`from_iter()`] push
/// onto the back in this manner, and iterating over `BoundedVecDeque` goes front to back.
///
/// This type is a wrapper around [`VecDeque`]. Almost all of its associated functions delegate to
/// `VecDeque`'s (after enforcing the length bound).
///
/// # Capacity and reallocation
///
/// At the time of writing, `VecDeque` behaves as follows:
///
/// * It always keeps its capacity at one less than a power of two.
/// * It always keeps an allocation (unlike e.g. `Vec`, where `new()` does not allocate and the
///   capacity can be reduced to zero).
/// * Its `reserve_exact()` is just an alias for `reserve()`.
///
/// This behavior is inherited by `BoundedVecDeque` (because it is merely a wrapper). It is not
/// documented by `VecDeque` (and is thus subject to change), but has been noted here because it
/// may be surprising or undesirable.
///
/// Users may wish to use maximum lengths that are one less than powers of two to prevent (at least
/// with the current `VecDeque` reallocation strategy) “wasted space” caused by the capacity
/// growing beyond the maximum length.
///
/// [`push_back()`]: #method.push_back
/// [`pop_front()`]: #method.pop_front
/// [`extend()`]: #method.extend
/// [`append()`]: #method.append
/// [`from_iter()`]: #method.from_iter
/// [`VecDeque`]: https://doc.rust-lang.org/std/collections/struct.VecDeque.html
#[derive(Debug)]
pub struct BoundedVecDeque<T> {
    vec_deque: VecDeque<T>,
    max_len: usize,
}

impl<T> BoundedVecDeque<T> {
    /// Creates a new, empty `BoundedVecDeque`.
    ///
    /// The capacity is set to the length limit (as a result, no reallocations will be necessary
    /// unless the length limit is later raised).
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let deque: BoundedVecDeque<i32> = BoundedVecDeque::new(255);
    ///
    /// assert!(deque.is_empty());
    /// assert_eq!(deque.max_len(), 255);
    /// assert!(deque.capacity() >= 255);
    /// ```
    pub fn new(max_len: usize) -> Self {
        BoundedVecDeque::with_capacity(max_len, max_len)
    }

    /// Creates a new, empty `BoundedVecDeque` with space for at least `capacity` elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let deque: BoundedVecDeque<i32> = BoundedVecDeque::with_capacity(63, 255);
    ///
    /// assert!(deque.is_empty());
    /// assert_eq!(deque.max_len(), 255);
    /// assert!(deque.capacity() >= 63);
    /// ```
    pub fn with_capacity(capacity: usize, max_len: usize) -> Self {
        BoundedVecDeque {
            vec_deque: VecDeque::with_capacity(capacity),
            max_len,
        }
    }

    /// Creates a new `BoundedVecDeque` from an iterator or iterable.
    ///
    /// At most `max_len` items are taken from the iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let five_fives = ::std::iter::repeat(5).take(5);
    ///
    /// let deque: BoundedVecDeque<i32> = BoundedVecDeque::from_iter(five_fives, 7);
    ///
    /// assert!(deque.iter().eq(&[5, 5, 5, 5, 5]));
    ///
    /// let mut numbers = 0..;
    ///
    /// let deque: BoundedVecDeque<i32> = BoundedVecDeque::from_iter(numbers.by_ref(), 7);
    ///
    /// assert!(deque.iter().eq(&[0, 1, 2, 3, 4, 5, 6]));
    /// assert_eq!(numbers.next(), Some(7));
    /// ```
    pub fn from_iter<I>(iterable: I, max_len: usize) -> Self
    where I: IntoIterator<Item=T> {
        BoundedVecDeque {
            vec_deque: iterable.into_iter().take(max_len).collect(),
            max_len,
        }
    }

    /// Creates a new `BoundedVecDeque` from a `VecDeque`.
    ///
    /// If `vec_deque` contains more than `max_len` items, excess items are dropped from the back.
    /// If the capacity is greater than `max_len`, it is [shrunk to fit].
    ///
    /// # Examples
    ///
    /// ```
    /// use ::std::collections::VecDeque;
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let unbounded = VecDeque::from(vec![42]);
    ///
    /// let bounded = BoundedVecDeque::from_unbounded(unbounded, 255);
    /// ```
    ///
    /// [shrunk to fit]: #method.shrink_to_fit
    pub fn from_unbounded(mut vec_deque: VecDeque<T>, max_len: usize) -> Self {
        vec_deque.truncate(max_len);
        if vec_deque.capacity() > max_len {
            vec_deque.shrink_to_fit();
        }
        BoundedVecDeque {
            vec_deque,
            max_len,
        }
    }

    /// Converts the `BoundedVecDeque` to `VecDeque`.
    ///
    /// This is a minimal-cost conversion.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let bounded = BoundedVecDeque::from_iter(vec![0, 1, 2, 3], 255);
    /// let unbounded = bounded.into_unbounded();
    /// ```
    pub fn into_unbounded(self) -> VecDeque<T> {
        self.vec_deque
    }

    /// Returns a mutable reference to an element in the `VecDeque` by index.
    ///
    /// Returns `None` if there is no such element.
    ///
    /// The element at index `0` is the front of the queue.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let mut deque = BoundedVecDeque::new(12);
    /// deque.push_back(3);
    /// deque.push_back(4);
    /// deque.push_back(5);
    ///
    /// if let Some(elem) = deque.get_mut(1) {
    ///     *elem = 7;
    /// }
    ///
    /// assert!(deque.iter().eq(&[3, 7, 5]));
    /// ```
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.vec_deque.get_mut(index)
    }

    /// Swaps the elements at indices `i` and `j`.
    ///
    /// `i` and `j` may be equal.
    ///
    /// The element at index `0` is the front of the queue.
    ///
    /// # Panics
    ///
    /// Panics if either index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let mut deque = BoundedVecDeque::new(12);
    /// deque.push_back(3);
    /// deque.push_back(4);
    /// deque.push_back(5);
    /// assert!(deque.iter().eq(&[3, 4, 5]));
    ///
    /// deque.swap(0, 2);
    ///
    /// assert!(deque.iter().eq(&[5, 4, 3]));
    /// ```
    pub fn swap(&mut self, i: usize, j: usize) {
        self.vec_deque.swap(i, j)
    }

    fn reserve_priv(&mut self, additional: usize, exact: bool) {
        let new_capacity = self.capacity()
                               .checked_add(additional)
                               .expect("capacity overflow");
        if new_capacity > self.max_len {
            panic!(
                "capacity out of bounds: the max len is {} but the new cap is {}",
                self.max_len,
                new_capacity,
            )
        }
        if exact {
            self.vec_deque.reserve_exact(additional)
        } else {
            self.vec_deque.reserve(additional)
        }
    }

    /// Reserves capacity for at least `additional` more elements to be inserted.
    ///
    /// To avoid frequent reallocations, more space than requested may be reserved.
    ///
    /// # Panics
    ///
    /// Panics if the requested new capacity exceeds the maximum length, or if the actual new
    /// capacity overflows `usize`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let mut deque = BoundedVecDeque::from_iter(vec![1], 255);
    ///
    /// deque.reserve(10);
    ///
    /// assert!(deque.capacity() >= 11);
    /// ```
    pub fn reserve(&mut self, additional: usize) {
        self.reserve_priv(additional, false)
    }

    /// Reserves the minimum capacity for exactly `additional` more elements to be inserted.
    ///
    /// Does nothing if the capacity is already sufficient.
    ///
    /// Note that the allocator may give the collection more space than it requests. Therefore
    /// capacity cannot be relied upon to be precisely minimal. Prefer [`reserve()`] if future
    /// insertions are expected.
    ///
    /// At the time of writing, **this method is equivalent to `reserve()`** because of
    /// `VecDeque`'s capacity management. It has been provided anyway for compatibility reasons.
    /// See [the relevant section of the type-level documentation][capacity] for details.
    ///
    /// # Panics
    ///
    /// Panics if the requested new capacity exceeds the maximum length, or if the actual new
    /// capacity overflows `usize`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let mut deque = BoundedVecDeque::from_iter(vec![1], 255);
    ///
    /// deque.reserve_exact(10);
    ///
    /// assert!(deque.capacity() >= 11);
    /// ```
    ///
    /// [`reserve()`]: #method.reserve
    /// [capacity]: #capacity-and-reallocation
    pub fn reserve_exact(&mut self, additional: usize) {
        self.reserve_priv(additional, true)
    }

    /// Reserves enough capacity for the collection to be filled to its maximum length.
    ///
    /// Does nothing if the capacity is already sufficient.
    ///
    /// See the [`reserve_exact()`] documentation for caveats about capacity management.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let mut deque = BoundedVecDeque::from_iter(vec![1], 255);
    ///
    /// deque.reserve_maximum();
    ///
    /// assert!(deque.capacity() >= 255);
    /// ```
    ///
    /// [`reserve_exact()`]: #method.reserve_exact
    pub fn reserve_maximum(&mut self) {
        let capacity = self.max_len().saturating_sub(self.len());
        self.vec_deque.reserve_exact(capacity)
    }

    /// Reduces the capacity as much as possible.
    ///
    /// The capacity is reduced to as close to the length as possible. However, [there are
    /// restrictions on how much the capacity can be reduced][capacity], and on top of that, the
    /// allocator may not shrink the allocation as much as `VecDeque` requests.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let mut deque = BoundedVecDeque::with_capacity(15, 15);
    /// deque.push_back(0);
    /// deque.push_back(1);
    /// deque.push_back(2);
    /// deque.push_back(3);
    /// assert_eq!(deque.capacity(), 15);
    ///
    /// deque.shrink_to_fit();
    ///
    /// assert!(deque.capacity() >= 4);
    /// ```
    ///
    /// [capacity]: #capacity-and-reallocation
    pub fn shrink_to_fit(&mut self) {
        self.vec_deque.shrink_to_fit()
    }

    /// Returns the maximum number of elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let deque: BoundedVecDeque<i32> = BoundedVecDeque::new(255);
    ///
    /// assert_eq!(deque.max_len(), 255);
    /// ```
    pub fn max_len(&self) -> usize {
        self.max_len
    }

    /// Changes the maximum number of elements to `max_len`.
    ///
    /// If there are more elements than the new maximum, they are removed from the back and yielded
    /// by the returned iterator (in front-to-back order).
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let mut deque: BoundedVecDeque<i32> = BoundedVecDeque::new(7);
    /// deque.extend(vec![0, 1, 2, 3, 4, 5, 6]);
    /// assert_eq!(deque.max_len(), 7);
    ///
    /// assert!(deque.set_max_len(3).eq(vec![3, 4, 5, 6]));
    ///
    /// assert_eq!(deque.max_len(), 3);
    /// assert!(deque.iter().eq(&[0, 1, 2]));
    /// ```
    pub fn set_max_len(&mut self, max_len: usize) -> Drain<'_, T> {
        let len = max_len.min(self.len());
        self.max_len = max_len;
        self.drain(len..)
    }

    /// Decreases the length, dropping excess elements from the back.
    ///
    /// If `new_len` is greater than the current length, this has no effect.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let mut deque = BoundedVecDeque::new(7);
    /// deque.push_back(5);
    /// deque.push_back(10);
    /// deque.push_back(15);
    /// assert!(deque.iter().eq(&[5, 10, 15]));
    ///
    /// deque.truncate(1);
    ///
    /// assert!(deque.iter().eq(&[5]));
    /// ```
    pub fn truncate(&mut self, new_len: usize) {
        self.vec_deque.truncate(new_len)
    }

    /// Returns a front-to-back iterator of immutable references.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let mut deque = BoundedVecDeque::new(7);
    /// deque.push_back(5);
    /// deque.push_back(3);
    /// deque.push_back(4);
    ///
    /// let deque_of_references: Vec<&i32> = deque.iter().collect();
    ///
    /// assert_eq!(&deque_of_references[..], &[&5, &3, &4]);
    /// ```
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            iter: self.vec_deque.iter(),
        }
    }

    /// Returns a front-to-back iterator of mutable references.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let mut deque = BoundedVecDeque::new(7);
    /// deque.push_back(5);
    /// deque.push_back(3);
    /// deque.push_back(4);
    ///
    /// for number in deque.iter_mut() {
    ///     *number -= 2;
    /// }
    /// let deque_of_references: Vec<&mut i32> = deque.iter_mut().collect();
    ///
    /// assert_eq!(&deque_of_references[..], &[&mut 3, &mut 1, &mut 2]);
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            iter: self.vec_deque.iter_mut(),
        }
    }

    /// Returns a reference to the underlying `VecDeque`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let bounded = BoundedVecDeque::from_iter(vec![0, 1, 2, 3], 255);
    /// let unbounded_ref = bounded.as_unbounded();
    /// ```
    pub fn as_unbounded(&self) -> &VecDeque<T> {
        self.as_ref()
    }

    /// Returns a pair of slices which contain the contents in order.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let mut deque = BoundedVecDeque::new(7);
    /// deque.push_back(0);
    /// deque.push_back(1);
    /// deque.push_front(10);
    /// deque.push_front(9);
    ///
    /// deque.as_mut_slices().0[0] = 42;
    /// deque.as_mut_slices().1[0] = 24;
    ///
    /// assert_eq!(deque.as_slices(), (&[42, 10][..], &[24, 1][..]));
    /// ```
    pub fn as_mut_slices(&mut self) -> (&mut [T], &mut [T]) {
        self.vec_deque.as_mut_slices()
    }

    /// Returns `true` if the `BoundedVecDeque` is full (and false otherwise).
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let mut deque = BoundedVecDeque::new(3);
    ///
    /// deque.push_back(0);
    /// assert!(!deque.is_full());
    /// deque.push_back(1);
    /// assert!(!deque.is_full());
    /// deque.push_back(2);
    /// assert!(deque.is_full());
    /// ```
    pub fn is_full(&self) -> bool {
        self.len() >= self.max_len
    }

    /// Creates a draining iterator that removes the specified range and yields the removed items.
    ///
    /// Note 1: The element range is removed even if the iterator is not consumed until the end.
    ///
    /// Note 2: It is unspecified how many elements are removed from the deque if the `Drain`
    /// value is not dropped but the borrow it holds expires (e.g. due to [`forget()`]).
    ///
    /// # Panics
    ///
    /// Panics if the start index is greater than the end index or if the end index is greater than
    /// the length of the deque.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let mut deque = BoundedVecDeque::from_iter(vec![0, 1, 2, 3], 7);
    ///
    /// assert!(deque.drain(2..).eq(vec![2, 3]));
    ///
    /// assert!(deque.iter().eq(&[0, 1]));
    ///
    /// // A full range clears all contents
    /// deque.drain(..);
    ///
    /// assert!(deque.is_empty());
    /// ```
    ///
    /// [`forget()`]: https://doc.rust-lang.org/std/mem/fn.forget.html
    pub fn drain<R>(&mut self, range: R) -> Drain<'_, T>
    where R: RangeBounds<usize> {
        Drain {
            iter: self.vec_deque.drain(range),
        }
    }

    /// Removes all values.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let mut deque = BoundedVecDeque::new(7);
    /// deque.push_back(0);
    /// assert!(!deque.is_empty());
    ///
    /// deque.clear();
    ///
    /// assert!(deque.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.vec_deque.clear()
    }

    /// Returns a mutable reference to the front element.
    ///
    /// Returns `None` if the deque is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let mut deque = BoundedVecDeque::new(7);
    ///
    /// assert_eq!(deque.front_mut(), None);
    ///
    /// deque.push_back(0);
    /// deque.push_back(1);
    ///
    /// if let Some(x) = deque.front_mut() {
    ///     *x = 9;
    /// }
    ///
    /// assert_eq!(deque.front(), Some(&9));
    /// ```
    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.vec_deque.front_mut()
    }

    /// Returns a mutable reference to the back element.
    ///
    /// Returns `None` if the deque is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let mut deque = BoundedVecDeque::new(7);
    ///
    /// assert_eq!(deque.back_mut(), None);
    ///
    /// deque.push_back(0);
    /// deque.push_back(1);
    ///
    /// if let Some(x) = deque.back_mut() {
    ///     *x = 9;
    /// }
    ///
    /// assert_eq!(deque.back(), Some(&9));
    /// ```
    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.vec_deque.back_mut()
    }

    /// Pushes an element onto the front of the deque.
    ///
    /// If the deque is full, an element is removed from the back and returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let mut deque = BoundedVecDeque::new(2);
    ///
    /// assert_eq!(deque.push_front(0), None);
    /// assert_eq!(deque.push_front(1), None);
    /// assert_eq!(deque.push_front(2), Some(0));
    /// assert_eq!(deque.push_front(3), Some(1));
    /// assert_eq!(deque.front(), Some(&3));
    /// ```
    pub fn push_front(&mut self, value: T) -> Option<T> {
        if self.max_len == 0 {
            return Some(value)
        }
        let displaced_value = if self.is_full() { self.pop_back() } else { None };
        self.vec_deque.push_front(value);
        displaced_value
    }

    /// Removes and returns the first element.
    ///
    /// Returns `None` if the deque is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let mut deque = BoundedVecDeque::new(7);
    /// deque.push_back(0);
    /// deque.push_back(1);
    ///
    /// assert_eq!(deque.pop_front(), Some(0));
    /// assert_eq!(deque.pop_front(), Some(1));
    /// assert_eq!(deque.pop_front(), None);
    /// ```
    pub fn pop_front(&mut self) -> Option<T> {
        self.vec_deque.pop_front()
    }

    /// Pushes an element onto the back of the deque.
    ///
    /// If the deque is full, an element is removed from the front and returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let mut deque = BoundedVecDeque::new(2);
    ///
    /// assert_eq!(deque.push_back(0), None);
    /// assert_eq!(deque.push_back(1), None);
    /// assert_eq!(deque.push_back(2), Some(0));
    /// assert_eq!(deque.push_back(3), Some(1));
    /// assert_eq!(deque.back(), Some(&3));
    /// ```
    pub fn push_back(&mut self, value: T) -> Option<T> {
        if self.max_len == 0 {
            return Some(value)
        }
        let displaced_value = if self.is_full() { self.pop_front() } else { None };
        self.vec_deque.push_back(value);
        displaced_value
    }

    /// Removes and returns the last element.
    ///
    /// Returns `None` if the deque is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let mut deque = BoundedVecDeque::new(7);
    /// deque.push_back(0);
    /// deque.push_back(1);
    ///
    /// assert_eq!(deque.pop_back(), Some(1));
    /// assert_eq!(deque.pop_back(), Some(0));
    /// assert_eq!(deque.pop_back(), None);
    /// ```
    pub fn pop_back(&mut self) -> Option<T> {
        self.vec_deque.pop_back()
    }

    /// Removes and returns the element at `index`, filling the gap with the element at the front.
    ///
    /// This does not preserve ordering, but is `O(1)`.
    ///
    /// Returns `None` if `index` is out of bounds.
    ///
    /// The element at index `0` is the front of the queue.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let mut deque = BoundedVecDeque::new(7);
    ///
    /// assert_eq!(deque.swap_remove_front(0), None);
    ///
    /// deque.extend(vec![0, 1, 2, 3, 4, 5, 6]);
    ///
    /// assert_eq!(deque.swap_remove_front(3), Some(3));
    /// assert!(deque.iter().eq(&[1, 2, 0, 4, 5, 6]));
    /// ```
    pub fn swap_remove_front(&mut self, index: usize) -> Option<T> {
        self.vec_deque.swap_remove_front(index)
    }

    /// Removes and returns the element at `index`, filling the gap with the element at the back.
    ///
    /// This does not preserve ordering, but is `O(1)`.
    ///
    /// Returns `None` if `index` is out of bounds.
    ///
    /// The element at index `0` is the front of the queue.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let mut deque = BoundedVecDeque::new(7);
    ///
    /// assert_eq!(deque.swap_remove_back(0), None);
    ///
    /// deque.extend(vec![0, 1, 2, 3, 4, 5, 6]);
    ///
    /// assert_eq!(deque.swap_remove_back(3), Some(3));
    /// assert!(deque.iter().eq(&[0, 1, 2, 6, 4, 5]));
    /// ```
    pub fn swap_remove_back(&mut self, index: usize) -> Option<T> {
        self.vec_deque.swap_remove_back(index)
    }

    /// Inserts an element at `index` in the deque, displacing the back if necessary.
    ///
    /// Elements with indices greater than or equal to `index` are shifted one place towards the
    /// back to make room. If the deque is full, an element is removed from the back and returned.
    ///
    /// The element at index `0` is the front of the queue.
    ///
    /// # Panics
    ///
    /// Panics if `index` is greater than the length.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let mut deque = BoundedVecDeque::new(5);
    /// deque.extend(vec!['a', 'b', 'c', 'd']);
    ///
    /// assert_eq!(deque.insert_spill_back(1, 'e'), None);
    /// assert!(deque.iter().eq(&['a', 'e', 'b', 'c', 'd']));
    /// assert_eq!(deque.insert_spill_back(1, 'f'), Some('d'));
    /// assert!(deque.iter().eq(&['a', 'f', 'e', 'b', 'c']));
    /// ```
    pub fn insert_spill_back(&mut self, index: usize, value: T) -> Option<T> {
        if self.max_len == 0 {
            return Some(value)
        }
        let displaced_value = if self.is_full() {
            self.pop_back()
        } else {
            None
        };
        self.vec_deque.insert(index, value);
        displaced_value
    }

    /// Inserts an element at `index` in the deque, displacing the front if necessary.
    ///
    /// If the deque is full, an element is removed from the front and returned, and elements with
    /// indices less than or equal to `index` are shifted one place towards the front to make room.
    /// Otherwise, elements with indices greater than or equal to `index` are shifted one place
    /// towards the back to make room.
    ///
    /// The element at index `0` is the front of the queue.
    ///
    /// # Panics
    ///
    /// Panics if `index` is greater than the length.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let mut deque = BoundedVecDeque::new(5);
    /// deque.extend(vec!['a', 'b', 'c', 'd']);
    ///
    /// assert_eq!(deque.insert_spill_front(3, 'e'), None);
    /// assert!(deque.iter().eq(&['a', 'b', 'c', 'e', 'd']));
    /// assert_eq!(deque.insert_spill_front(3, 'f'), Some('a'));
    /// assert!(deque.iter().eq(&['b', 'c', 'e', 'f', 'd']));
    /// ```
    pub fn insert_spill_front(&mut self, index: usize, value: T) -> Option<T> {
        if self.max_len == 0 {
            return Some(value)
        }
        let displaced_value = if self.is_full() {
            self.pop_front()
        } else {
            None
        };
        self.vec_deque.insert(index, value);
        displaced_value
    }

    /// Removes and returns the element at `index`.
    ///
    /// Elements with indices greater than `index` are shifted towards the front to fill the gap.
    ///
    /// Returns `None` if `index` is out of bounds.
    ///
    /// The element at index `0` is the front of the queue.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let mut deque = BoundedVecDeque::new(7);
    ///
    /// assert_eq!(deque.remove(0), None);
    ///
    /// deque.extend(vec![0, 1, 2, 3, 4, 5, 6]);
    ///
    /// assert_eq!(deque.remove(3), Some(3));
    /// assert!(deque.iter().eq(&[0, 1, 2, 4, 5, 6]));
    /// ```
    pub fn remove(&mut self, index: usize) -> Option<T> {
        self.vec_deque.remove(index)
    }

    /// Splits the deque in two at the given index.
    ///
    /// Returns a new `BoundedVecDeque` containing elements `[at, len)`, leaving `self` with
    /// elements `[0, at)`. The capacity and maximum length of `self` are unchanged, and the new
    /// deque has the same maximum length as `self`.
    ///
    /// The element at index `0` is the front of the queue.
    ///
    /// # Panics
    ///
    /// Panics if `at > len`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let mut deque = BoundedVecDeque::from_iter(vec![0, 1, 2, 3], 7);
    ///
    /// let other_deque = deque.split_off(2);
    ///
    /// assert!(other_deque.iter().eq(&[2, 3]));
    /// assert!(deque.iter().eq(&[0, 1]));
    /// ```
    pub fn split_off(&mut self, at: usize) -> Self {
        BoundedVecDeque {
            vec_deque: self.vec_deque.split_off(at),
            max_len: self.max_len,
        }
    }

    /// Moves all the elements of `other` into `self`, leaving `other` empty.
    ///
    /// Elements from `other` are pushed onto the back of `self`. If the maximum length is
    /// exceeded, excess elements from the front of `self` are yielded by the returned iterator.
    ///
    /// # Panics
    ///
    /// Panics if the new number of elements in self overflows a `usize`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let mut deque = BoundedVecDeque::from_iter(vec![0, 1, 2, 3], 7);
    /// let mut other_deque = BoundedVecDeque::from_iter(vec![4, 5, 6, 7, 8], 7);
    ///
    /// assert!(deque.append(&mut other_deque).eq(vec![0, 1]));
    ///
    /// assert!(deque.iter().eq(&[2, 3, 4, 5, 6, 7, 8]));
    /// assert!(other_deque.is_empty());
    /// ```
    pub fn append<'a>(&'a mut self, other: &'a mut Self) -> Append<'a, T> {
        Append {
            source: other,
            destination: self,
        }
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// `predicate` is called for each element; each element for which it returns `false` is
    /// removed. This method operates in place and preserves the order of the retained elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let mut deque = BoundedVecDeque::from_iter(1..5, 7);
    ///
    /// deque.retain(|&x| x % 2 == 0);
    ///
    /// assert!(deque.iter().eq(&[2, 4]));
    /// ```
    pub fn retain<F>(&mut self, predicate: F)
    where F: FnMut(&T) -> bool {
        self.vec_deque.retain(predicate)
    }

    /// Modifies the deque in-place so that its length is equal to `new_len`.
    ///
    /// This is done either by removing excess elements from the back or by pushing clones of
    /// `value` to the back.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let mut deque = BoundedVecDeque::from_iter(vec![5, 10, 15], 7);
    ///
    /// deque.resize(2, 0);
    ///
    /// assert!(deque.iter().eq(&[5, 10]));
    ///
    /// deque.resize(5, 20);
    ///
    /// assert!(deque.iter().eq(&[5, 10, 20, 20, 20]));
    /// ```
    pub fn resize(&mut self, new_len: usize, value: T)
    where T: Clone {
        if new_len > self.max_len {
            panic!(
                "length out of bounds: the new len is {} but the max len is {}",
                new_len,
                self.max_len,
            )
        }
        self.vec_deque.resize(new_len, value)
    }

    /// Modifies the deque in-place so that its length is equal to `new_len`.
    ///
    /// This is done either by removing excess elements from the back or by pushing elements
    /// produced by calling `producer` to the back.
    ///
    /// # Availability
    ///
    /// This method requires [the `resize_with` feature], which requires Rust 1.33.
    ///
    /// [the `resize_with` feature]: index.html#features
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let mut deque = BoundedVecDeque::from_iter(vec![5, 10, 15], 7);
    ///
    /// deque.resize_with(5, Default::default);
    /// assert!(deque.iter().eq(&[5, 10, 15, 0, 0]));
    ///
    /// deque.resize_with(2, || unreachable!());
    /// assert!(deque.iter().eq(&[5, 10]));
    ///
    /// let mut state = 100;
    /// deque.resize_with(5, || { state += 1; state });
    /// assert!(deque.iter().eq(&[5, 10, 101, 102, 103]));
    /// ```
    #[cfg(feature = "resize_with")]
    pub fn resize_with<F>(&mut self, new_len: usize, producer: F)
    where F: FnMut() -> T {
        if new_len > self.max_len {
            panic!(
                "length out of bounds: the new len is {} but the max len is {}",
                new_len,
                self.max_len,
            )
        }
        self.vec_deque.resize_with(new_len, producer)
    }
}

impl<T: Clone> Clone for BoundedVecDeque<T> {
    fn clone(&self) -> Self {
        BoundedVecDeque {
            vec_deque: self.vec_deque.clone(),
            max_len: self.max_len,
        }
    }

    /// Mutates `self` into a clone of `other` (like `*self = other.clone()`).
    ///
    /// `self` is cleared, and the elements of `other` are cloned and added. The maximum length is
    /// set to the same as `other`'s.
    ///
    /// This method reuses `self`'s allocation, but due to API limitations, the allocation cannot
    /// be shrunk to fit the maximum length. Because of this, if `self`'s capacity is more than the
    /// new maximum length, it is shrunk to fit _`other`'s_ length.
    fn clone_from(&mut self, other: &Self) {
        self.clear();
        self.max_len = other.max_len;
        let should_shrink = self.capacity() > self.max_len;
        if should_shrink {
            self.reserve_exact(other.len());
        } else {
            self.reserve(other.len());
        }
        self.extend(other.iter().cloned());
        if should_shrink {
            // Ideally, we would shrink to self.max_len, and do so _before_ pushing all the cloned
            // values, but shrink_to() isn't stable yet.
            self.shrink_to_fit();
        }
    }
}

impl<T: Hash> Hash for BoundedVecDeque<T> {
    /// Feeds `self` into `hasher`.
    ///
    /// Only the values contained in `self` are hashed; the length bound is ignored.
    fn hash<H>(&self, hasher: &mut H)
    where H: Hasher {
        self.vec_deque.hash(hasher)
    }
}

impl<T> Deref for BoundedVecDeque<T> {
    type Target = VecDeque<T>;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> AsRef<VecDeque<T>> for BoundedVecDeque<T> {
    fn as_ref(&self) -> &VecDeque<T> {
        &self.vec_deque
    }
}

impl<T> Index<usize> for BoundedVecDeque<T> {
    type Output = T;

    /// Returns a reference to an element in the `VecDeque` by index.
    ///
    /// The element at index `0` is the front of the queue.
    ///
    /// # Panics
    ///
    /// Panics if there is no such element (i.e. `index >= len`).
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let mut deque = BoundedVecDeque::new(7);
    /// deque.push_back(3);
    /// deque.push_back(4);
    /// deque.push_back(5);
    ///
    /// let value = &deque[1];
    ///
    /// assert_eq!(value, &4);
    /// ```
    fn index(&self, index: usize) -> &T {
        &self.vec_deque[index]
    }
}

impl<T> IndexMut<usize> for BoundedVecDeque<T> {
    /// Returns a mutable reference to an element in the `VecDeque` by index.
    ///
    /// The element at index `0` is the front of the queue.
    ///
    /// # Panics
    ///
    /// Panics if there is no such element (i.e. `index >= len`).
    ///
    /// # Examples
    ///
    /// ```
    /// use ::bounded_vec_deque::BoundedVecDeque;
    ///
    /// let mut deque = BoundedVecDeque::new(12);
    /// deque.push_back(3);
    /// deque.push_back(4);
    /// deque.push_back(5);
    ///
    /// deque[1] = 7;
    ///
    /// assert_eq!(deque[1], 7);
    /// ```
    fn index_mut(&mut self, index: usize) -> &mut T {
        &mut self.vec_deque[index]
    }
}

impl<T> IntoIterator for BoundedVecDeque<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            iter: self.vec_deque.into_iter(),
        }
    }
}

impl<'a, T> IntoIterator for &'a BoundedVecDeque<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut BoundedVecDeque<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> Extend<T> for BoundedVecDeque<T> {
    fn extend<I>(&mut self, iter: I)
    where I: IntoIterator<Item=T> {
        for value in iter {
            self.push_back(value);
        }
    }
}
