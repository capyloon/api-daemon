#[cfg(any(feature = "alloc", feature = "std"))]
mod boxed;

#[cfg(any(feature = "alloc", feature = "std"))]
mod concat;

#[cfg(any(feature = "alloc", feature = "std"))]
mod join;

mod slice_index;

use core::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops,
    slice::{
        Chunks, ChunksExact, ChunksExactMut, ChunksMut, Iter, IterMut, RChunks, RChunksExact,
        RChunksExactMut, RChunksMut, RSplit, RSplitMut, RSplitN, RSplitNMut, Split, SplitMut,
        SplitN, SplitNMut, Windows,
    },
};

#[cfg(any(feature = "alloc", feature = "std"))]
use alloc::{borrow::ToOwned, boxed::Box};

#[cfg(feature = "serde")]
use serde::ser::{Serialize, Serializer};

use crate::{TiEnumerated, TiRangeBounds, TiSliceKeys, TiSliceMutMap, TiSliceRefMap};

#[cfg(any(feature = "alloc", feature = "std"))]
use crate::TiVec;

#[cfg(any(feature = "alloc", feature = "std"))]
use concat::Concat;

#[cfg(any(feature = "alloc", feature = "std"))]
use join::Join;

pub use slice_index::TiSliceIndex;

/// A dynamically-sized view into a contiguous sequence of `T`
/// that only accepts keys of the type `K`.
///
/// `TiSlice<K, V>` is a wrapper around Rust primitive type [`slice`].
/// The struct mirrors the stable API of Rust [`slice`]
/// and forwards to it as much as possible.
///
/// `TiSlice<K, V>` uses `K` instead of `usize` for element indices.
/// It also uses [`Range`], [`RangeTo`], [`RangeFrom`], [`RangeInclusive`] and
/// [`RangeToInclusive`] range types with `K` indices for `get`-methods and index expressions.
/// The [`RangeFull`] trait is not currently supported.
///
/// `TiSlice<K, V>` require the index to implement
/// [`From<usize>`][`From`] and [`Into<usize>`][`Into`] traits.
/// Their implementation can be easily done
/// with [`derive_more`] crate and `#[derive(From, Into)]`.
///
/// There are zero-cost conversions available between types and references:
/// - [`&[V]`][`slice`] and `&TiSlice<K, V>` with [`AsRef`],
/// - [`&mut [V]`][`slice`] and `&mut TiSlice<K, V>` with [`AsMut`],
/// - [`Box<[V]>`][`Box`] and `Box<TiSlice<K, V>>` with [`From`] and [`Into`].
///
/// Added methods:
/// - [`from_ref`] - Converts a [`&[V]`][`slice`] into a `&TiSlice<K, V>`.
/// - [`from_mut`] - Converts a [`&mut [V]`][`slice`] into a `&mut TiSlice<K, V>`.
/// - [`keys`] - Returns an iterator over all keys.
/// - [`next_key`] - Returns the index of the next slice element to be appended
///   and at the same time number of elements in the slice of type `K`.
/// - [`first_key`] - Returns the first slice element index of type `K`,
///   or `None` if the slice is empty.
/// - [`first_key_value`] - Returns the first slice element index of type `K`
///   and the element itself, or `None` if the slice is empty.
/// - [`first_key_value_mut`] - Returns the first slice element index of type `K`
///   and a mutable reference to the element itself, or `None` if the slice is empty.
/// - [`last_key`] - Returns the last slice element index of type `K`,
///   or `None` if the slice is empty.
/// - [`last_key_value`] - Returns the last slice element index of type `K`
///   and the element itself, or `None` if the slice is empty.
/// - [`last_key_value_mut`] - Returns the last slice element index of type `K`
///   and a mutable reference to the element itself, or `None` if the slice is empty.
/// - [`iter_enumerated`] - Returns an iterator over all key-value pairs.
///   It acts like `self.iter().enumerate()`,
///   but use `K` instead of `usize` for iteration indices.
/// - [`iter_mut_enumerated`] - Returns an iterator over all key-value pairs,
///   with mutable references to the values.
///   It acts like `self.iter_mut().enumerate()`,
///   but use `K` instead of `usize` for iteration indices.
/// - [`position`] - Searches for an element in an iterator, returning its index of type `K`.
///   It acts like `self.iter().position(...)`,
///   but instead of `usize` it returns index of type `K`.
/// - [`rposition`] - Searches for an element in an iterator from the right,
/// returning its index of type `K`.
///   It acts like `self.iter().rposition(...)`,
///   but instead of `usize` it returns index of type `K`.
///
/// # Example
///
/// ```
/// use typed_index_collections::TiSlice;
/// use derive_more::{From, Into};
///
/// #[derive(From, Into)]
/// struct FooId(usize);
///
/// let mut foos_raw = [1, 2, 5, 8];
/// let foos: &mut TiSlice<FooId, usize> = TiSlice::from_mut(&mut foos_raw);
/// foos[FooId(2)] = 4;
/// assert_eq!(foos[FooId(2)], 4);
/// ```
///
/// [`from_ref`]: #method.from_ref
/// [`from_mut`]: #method.from_mut
/// [`keys`]: #method.keys
/// [`next_key`]: #method.next_key
/// [`first_key`]: #method.first_key
/// [`first_key_value`]: #method.first_key_value
/// [`first_key_value_mut`]: #method.first_key_value_mut
/// [`last_key`]: #method.last_key
/// [`last_key_value`]: #method.last_key_value
/// [`last_key_value_mut`]: #method.last_key_value_mut
/// [`iter_enumerated`]: #method.iter_enumerated
/// [`iter_mut_enumerated`]: #method.iter_mut_enumerated
/// [`position`]: #method.position
/// [`rposition`]: #method.rposition
/// [`slice`]: https://doc.rust-lang.org/std/primitive.slice.html
/// [`From`]: https://doc.rust-lang.org/std/convert/trait.From.html
/// [`Into`]: https://doc.rust-lang.org/std/convert/trait.Into.html
/// [`AsRef`]: https://doc.rust-lang.org/std/convert/trait.AsRef.html
/// [`AsMut`]: https://doc.rust-lang.org/std/convert/trait.AsMut.html
/// [`Box`]: https://doc.rust-lang.org/std/boxed/struct.Box.html
/// [`Range`]: https://doc.rust-lang.org/std/ops/struct.Range.html
/// [`RangeTo`]: https://doc.rust-lang.org/std/ops/struct.RangeTo.html
/// [`RangeFrom`]: https://doc.rust-lang.org/std/ops/struct.RangeFrom.html
/// [`RangeInclusive`]: https://doc.rust-lang.org/std/ops/struct.RangeInclusive.html
/// [`RangeToInclusive`]: https://doc.rust-lang.org/std/ops/struct.RangeToInclusive.html
/// [`RangeFull`]: https://doc.rust-lang.org/std/ops/struct.RangeFull.html
/// [`derive_more`]: https://crates.io/crates/derive_more
#[repr(transparent)]
pub struct TiSlice<K, V> {
    /// Tied slice index type
    ///
    /// `fn(T) -> T` is *[PhantomData pattern][phantomdata patterns]*
    /// used to relax auto trait implementations bounds for
    /// [`Send`], [`Sync`], [`Unpin`], [`UnwindSafe`] and [`RefUnwindSafe`].
    ///
    /// Derive attribute is not used for trait implementations because it also requires
    /// the same trait implemented for K that is an unnecessary requirement.
    ///
    /// [phantomdata patterns]: https://doc.rust-lang.org/nomicon/phantom-data.html#table-of-phantomdata-patterns
    /// [`Send`]: https://doc.rust-lang.org/core/marker/trait.Send.html
    /// [`Sync`]: https://doc.rust-lang.org/core/marker/trait.Sync.html
    /// [`Unpin`]: https://doc.rust-lang.org/core/marker/trait.Unpin.html
    /// [`UnwindSafe`]: https://doc.rust-lang.org/core/std/panic/trait.UnwindSafe.html
    /// [`RefUnwindSafe`]: https://doc.rust-lang.org/core/std/panic/trait.RefUnwindSafe.html
    _marker: PhantomData<fn(K) -> K>,

    /// Raw slice property
    pub raw: [V],
}

impl<K, V> TiSlice<K, V> {
    /// Converts a `&[V]` into a `&TiSlice<K, V>`.
    ///
    /// # Example
    ///
    /// ```
    /// # use typed_index_collections::TiSlice;
    /// pub struct Id(usize);
    /// let slice: &TiSlice<Id, usize> = TiSlice::from_ref(&[1, 2, 4]);
    /// ```
    #[allow(trivial_casts)]
    #[inline]
    pub fn from_ref(raw: &[V]) -> &Self {
        unsafe { &*(raw as *const [V] as *const Self) }
    }

    /// Converts a `&mut [V]` into a `&mut TiSlice<K, V>`.
    ///
    /// # Example
    ///
    /// ```
    /// # use typed_index_collections::TiSlice;
    /// pub struct Id(usize);
    /// let slice: &mut TiSlice<Id, usize> = TiSlice::from_mut(&mut [1, 2, 4]);
    /// ```
    #[allow(trivial_casts)]
    #[inline]
    pub fn from_mut(raw: &mut [V]) -> &mut Self {
        unsafe { &mut *(raw as *mut [V] as *mut Self) }
    }

    /// Returns the number of elements in the slice.
    ///
    /// See [`slice::len`] for more details.
    ///
    /// [`slice::len`]: https://doc.rust-lang.org/std/primitive.slice.html#method.len
    #[inline]
    pub fn len(&self) -> usize {
        self.raw.len()
    }

    /// Returns the index of the next slice element to be appended
    /// and at the same time number of elements in the slice of type `K`.
    ///
    /// # Example
    ///
    /// ```
    /// # use derive_more::{From, Into};
    /// # use typed_index_collections::TiSlice;
    /// #[derive(Eq, Debug, From, Into, PartialEq)]
    /// pub struct Id(usize);
    /// let slice: &TiSlice<Id, usize> = TiSlice::from_ref(&[1, 2, 4]);
    /// assert_eq!(slice.next_key(), Id(3));
    /// ```
    #[inline]
    pub fn next_key(&self) -> K
    where
        K: From<usize>,
    {
        self.raw.len().into()
    }

    /// Returns `true` if the slice has a length of 0.
    ///
    /// See [`slice::is_empty`] for more details.
    ///
    /// [`slice::is_empty`]: https://doc.rust-lang.org/std/primitive.slice.html#method.is_empty
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    /// Returns an iterator over all keys.
    ///
    /// # Example
    ///
    /// ```
    /// # use derive_more::{From, Into};
    /// # use typed_index_collections::TiSlice;
    /// #[derive(Debug, Eq, From, Into, PartialEq)]
    /// pub struct Id(usize);
    /// let slice: &TiSlice<Id, usize> = TiSlice::from_ref(&[1, 2, 4]);
    /// let mut iterator = slice.keys();
    /// assert_eq!(iterator.next(), Some(Id(0)));
    /// assert_eq!(iterator.next(), Some(Id(1)));
    /// assert_eq!(iterator.next(), Some(Id(2)));
    /// assert_eq!(iterator.next(), None);
    /// ```
    pub fn keys(&self) -> TiSliceKeys<K>
    where
        K: From<usize>,
    {
        (0..self.len()).map(Into::into)
    }

    /// Returns the first element of the slice, or `None` if it is empty.
    ///
    /// See [`slice::first`] for more details.
    ///
    /// [`slice::first`]: https://doc.rust-lang.org/std/primitive.slice.html#method.first
    #[inline]
    pub fn first(&self) -> Option<&V> {
        self.raw.first()
    }

    /// Returns a mutable reference to the first element of the slice, or `None` if it is empty.
    ///
    /// See [`slice::first_mut`] for more details.
    ///
    /// [`slice::first_mut`]: https://doc.rust-lang.org/std/primitive.slice.html#method.first_mut
    #[inline]
    pub fn first_mut(&mut self) -> Option<&mut V> {
        self.raw.first_mut()
    }

    /// Returns the first slice element index of type `K`, or `None` if the slice is empty.
    ///
    /// # Example
    ///
    /// ```
    /// # use derive_more::{From, Into};
    /// # use typed_index_collections::TiSlice;
    /// #[derive(Debug, Eq, From, Into, PartialEq)]
    /// pub struct Id(usize);
    /// let empty_slice: &TiSlice<Id, usize> = TiSlice::from_ref(&[]);
    /// let slice: &TiSlice<Id, usize> = TiSlice::from_ref(&[1, 2, 4]);
    /// assert_eq!(empty_slice.first_key(), None);
    /// assert_eq!(slice.first_key(), Some(Id(0)));
    /// ```
    #[inline]
    pub fn first_key(&self) -> Option<K>
    where
        K: From<usize>,
    {
        if self.is_empty() {
            None
        } else {
            Some(0.into())
        }
    }

    /// Returns the first slice element index of type `K` and the element itself,
    /// or `None` if the slice is empty.
    ///
    /// See [`slice::first`] for more details.
    ///
    /// # Example
    ///
    /// ```
    /// # use derive_more::{From, Into};
    /// # use typed_index_collections::TiSlice;
    /// #[derive(Debug, Eq, From, Into, PartialEq)]
    /// pub struct Id(usize);
    /// let empty_slice: &TiSlice<Id, usize> = TiSlice::from_ref(&[]);
    /// let slice: &TiSlice<Id, usize> = TiSlice::from_ref(&[1, 2, 4]);
    /// assert_eq!(empty_slice.first_key_value(), None);
    /// assert_eq!(slice.first_key_value(), Some((Id(0), &1)));
    /// ```
    ///
    /// [`slice::first`]: https://doc.rust-lang.org/std/primitive.slice.html#method.first
    #[inline]
    pub fn first_key_value(&self) -> Option<(K, &V)>
    where
        K: From<usize>,
    {
        self.raw.first().map(|first| (0.into(), first))
    }

    /// Returns the first slice element index of type `K` and a mutable reference
    /// to the element itself, or `None` if the slice is empty.
    ///
    /// See [`slice::first_mut`] for more details.
    ///
    /// # Example
    ///
    /// ```
    /// # use derive_more::{From, Into};
    /// # use typed_index_collections::TiSlice;
    /// #[derive(Debug, Eq, From, Into, PartialEq)]
    /// pub struct Id(usize);
    /// let empty_slice: &mut TiSlice<Id, usize> = TiSlice::from_mut(&mut []);
    /// let mut array = [1, 2, 4];
    /// let slice: &mut TiSlice<Id, usize> = TiSlice::from_mut(&mut array);
    /// assert_eq!(empty_slice.first_key_value_mut(), None);
    /// assert_eq!(slice.first_key_value_mut(), Some((Id(0), &mut 1)));
    /// *slice.first_key_value_mut().unwrap().1 = 123;
    /// assert_eq!(slice.raw, [123, 2, 4]);
    /// ```
    ///
    /// [`slice::first_mut`]: https://doc.rust-lang.org/std/primitive.slice.html#method.first_mut
    #[inline]
    pub fn first_key_value_mut(&mut self) -> Option<(K, &mut V)>
    where
        K: From<usize>,
    {
        self.raw.first_mut().map(|first| (0.into(), first))
    }

    /// Returns the first and all the rest of the elements of the slice, or `None` if it is empty.
    ///
    /// See [`slice::split_first`] for more details.
    ///
    /// [`slice::split_first`]: https://doc.rust-lang.org/std/primitive.slice.html#method.split_first
    #[inline]
    pub fn split_first(&self) -> Option<(&V, &TiSlice<K, V>)> {
        self.raw
            .split_first()
            .map(|(first, rest)| (first, rest.as_ref()))
    }

    /// Returns the first and all the rest of the elements of the slice, or `None` if it is empty.
    ///
    /// See [`slice::split_first_mut`] for more details.
    ///
    /// [`slice::split_first_mut`]: https://doc.rust-lang.org/std/primitive.slice.html#method.split_first_mut
    #[inline]
    pub fn split_first_mut(&mut self) -> Option<(&mut V, &mut TiSlice<K, V>)> {
        self.raw
            .split_first_mut()
            .map(|(first, rest)| (first, rest.as_mut()))
    }

    /// Returns the last and all the rest of the elements of the slice, or `None` if it is empty.
    ///
    /// See [`slice::split_last`] for more details.
    ///
    /// [`slice::split_last`]: https://doc.rust-lang.org/std/primitive.slice.html#method.split_last
    #[inline]
    pub fn split_last(&self) -> Option<(&V, &TiSlice<K, V>)> {
        self.raw
            .split_last()
            .map(|(last, rest)| (last, rest.as_ref()))
    }

    /// Returns the last and all the rest of the elements of the slice, or `None` if it is empty.
    ///
    /// See [`slice::split_last_mut`] for more details.
    ///
    /// [`slice::split_last_mut`]: https://doc.rust-lang.org/std/primitive.slice.html#method.split_last_mut
    #[inline]
    pub fn split_last_mut(&mut self) -> Option<(&mut V, &mut TiSlice<K, V>)> {
        self.raw
            .split_last_mut()
            .map(|(last, rest)| (last, rest.as_mut()))
    }

    /// Returns the last element of the slice of type `K`, or `None` if it is empty.
    ///
    /// See [`slice::last`] for more details.
    ///
    /// [`slice::last`]: https://doc.rust-lang.org/std/primitive.slice.html#method.last
    #[inline]
    pub fn last(&self) -> Option<&V> {
        self.raw.last()
    }

    /// Returns a mutable reference to the last item in the slice.
    ///
    /// See [`slice::last_mut`] for more details.
    ///
    /// [`slice::last_mut`]: https://doc.rust-lang.org/std/primitive.slice.html#method.last_mut
    #[inline]
    pub fn last_mut(&mut self) -> Option<&mut V> {
        self.raw.last_mut()
    }

    /// Returns the last slice element index of type `K`, or `None` if the slice is empty.
    ///
    /// # Example
    ///
    /// ```
    /// # use derive_more::{From, Into};
    /// # use typed_index_collections::TiSlice;
    /// #[derive(Debug, Eq, From, Into, PartialEq)]
    /// pub struct Id(usize);
    /// let empty_slice: &TiSlice<Id, usize> = TiSlice::from_ref(&[]);
    /// let slice: &TiSlice<Id, usize> = TiSlice::from_ref(&[1, 2, 4]);
    /// assert_eq!(empty_slice.last_key(), None);
    /// assert_eq!(slice.last_key(), Some(Id(2)));
    /// ```
    #[inline]
    pub fn last_key(&self) -> Option<K>
    where
        K: From<usize>,
    {
        if self.is_empty() {
            None
        } else {
            Some((self.len() - 1).into())
        }
    }

    /// Returns the last slice element index of type `K` and the element itself,
    /// or `None` if the slice is empty.
    ///
    /// See [`slice::last`] for more details.
    ///
    /// # Example
    ///
    /// ```
    /// # use derive_more::{From, Into};
    /// # use typed_index_collections::TiSlice;
    /// #[derive(Debug, Eq, From, Into, PartialEq)]
    /// pub struct Id(usize);
    /// let empty_slice: &TiSlice<Id, usize> = TiSlice::from_ref(&[]);
    /// let slice: &TiSlice<Id, usize> = TiSlice::from_ref(&[1, 2, 4]);
    /// assert_eq!(empty_slice.last_key_value(), None);
    /// assert_eq!(slice.last_key_value(), Some((Id(2), &4)));
    /// ```
    ///
    /// [`slice::last`]: https://doc.rust-lang.org/std/primitive.slice.html#method.last
    #[inline]
    pub fn last_key_value(&self) -> Option<(K, &V)>
    where
        K: From<usize>,
    {
        let len = self.len();
        self.raw.last().map(|last| ((len - 1).into(), last))
    }

    /// Returns the last slice element index of type `K` and a mutable reference
    /// to the element itself, or `None` if the slice is empty.
    ///
    /// See [`slice::last_mut`] for more details.
    ///
    /// # Example
    ///
    /// ```
    /// # use derive_more::{From, Into};
    /// # use typed_index_collections::TiSlice;
    /// #[derive(Debug, Eq, From, Into, PartialEq)]
    /// pub struct Id(usize);
    /// let empty_slice: &mut TiSlice<Id, usize> = TiSlice::from_mut(&mut []);
    /// let mut array = [1, 2, 4];
    /// let slice: &mut TiSlice<Id, usize> = TiSlice::from_mut(&mut array);
    /// assert_eq!(empty_slice.last_key_value_mut(), None);
    /// assert_eq!(slice.last_key_value_mut(), Some((Id(2), &mut 4)));
    /// *slice.last_key_value_mut().unwrap().1 = 123;
    /// assert_eq!(slice.raw, [1, 2, 123]);
    /// ```
    ///
    /// [`slice::last_mut`]: https://doc.rust-lang.org/std/primitive.slice.html#method.last_mut
    #[inline]
    pub fn last_key_value_mut(&mut self) -> Option<(K, &mut V)>
    where
        K: From<usize>,
    {
        let len = self.len();
        self.raw.last_mut().map(|last| ((len - 1).into(), last))
    }

    /// Returns a reference to an element or subslice
    /// depending on the type of index or `None` if the index is out of bounds.
    ///
    /// See [`slice::get`] for more details.
    ///
    /// [`slice::get`]: https://doc.rust-lang.org/std/primitive.slice.html#method.get
    #[inline]
    pub fn get<I>(&self, index: I) -> Option<&I::Output>
    where
        I: TiSliceIndex<K, V>,
    {
        index.get(self)
    }

    /// Returns a mutable reference to an element or subslice
    /// depending on the type of index or `None` if the index is out of bounds.
    ///
    /// See [`slice::get_mut`] for more details.
    ///
    /// [`slice::get_mut`]: https://doc.rust-lang.org/std/primitive.slice.html#method.get_mut
    #[inline]
    pub fn get_mut<I>(&mut self, index: I) -> Option<&mut I::Output>
    where
        I: TiSliceIndex<K, V>,
    {
        index.get_mut(self)
    }

    /// Returns a reference to an element or subslice
    /// depending on the type of index, without doing bounds checking.
    ///
    /// This is generally not recommended, use with caution!
    /// Calling this method with an out-of-bounds index is *[undefined behavior]*
    /// even if the resulting reference is not used.
    /// For a safe alternative see [`get`].
    ///
    /// See [`slice::get_unchecked`] for more details.
    ///
    /// [`get`]: #method.get
    /// [`slice::get_unchecked`]: https://doc.rust-lang.org/std/primitive.slice.html#method.get_unchecked
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    #[allow(clippy::missing_safety_doc)]
    #[inline]
    pub unsafe fn get_unchecked<I>(&self, index: I) -> &I::Output
    where
        I: TiSliceIndex<K, V>,
    {
        index.get_unchecked(self)
    }

    /// Returns a mutable reference to an element or subslice
    /// depending on the type of index, without doing bounds checking.
    ///
    /// This is generally not recommended, use with caution!
    /// Calling this method with an out-of-bounds index is *[undefined behavior]*
    /// even if the resulting reference is not used.
    /// For a safe alternative see [`get_mut`].
    ///
    /// See [`slice::get_unchecked_mut`] for more details.
    ///
    /// [`get_mut`]: #method.get_mut
    /// [`slice::get_unchecked_mut`]: https://doc.rust-lang.org/std/primitive.slice.html#method.get_unchecked_mut
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    #[allow(clippy::missing_safety_doc)]
    #[inline]
    pub unsafe fn get_unchecked_mut<I>(&mut self, index: I) -> &mut I::Output
    where
        I: TiSliceIndex<K, V>,
    {
        index.get_unchecked_mut(self)
    }

    /// Returns a raw pointer to the slice's buffer.
    ///
    /// See [`slice::as_ptr`] for more details.
    ///
    /// [`slice::as_ptr`]: https://doc.rust-lang.org/std/primitive.slice.html#method.as_ptr
    #[inline]
    pub const fn as_ptr(&self) -> *const V {
        self.raw.as_ptr()
    }

    /// Returns an unsafe mutable reference to the slice's buffer.
    ///
    /// See [`slice::as_mut_ptr`] for more details.
    ///
    /// [`slice::as_mut_ptr`]: https://doc.rust-lang.org/std/primitive.slice.html#method.as_mut_ptr
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut V {
        self.raw.as_mut_ptr()
    }

    /// Swaps two elements in the slice.
    ///
    /// See [`slice::swap`] for more details.
    ///
    /// [`slice::swap`]: https://doc.rust-lang.org/std/primitive.slice.html#method.swap
    #[inline]
    pub fn swap(&mut self, a: K, b: K)
    where
        usize: From<K>,
    {
        self.raw.swap(a.into(), b.into())
    }

    /// Reverses the order of elements in the slice, in place.
    ///
    /// See [`slice::reverse`] for more details.
    ///
    /// [`slice::reverse`]: https://doc.rust-lang.org/std/primitive.slice.html#method.reverse
    #[inline]
    pub fn reverse(&mut self) {
        self.raw.reverse()
    }

    /// Returns an iterator over the slice.
    ///
    /// See [`slice::iter`] for more details.
    ///
    /// [`slice::iter`]: https://doc.rust-lang.org/std/primitive.slice.html#method.iter
    #[inline]
    pub fn iter(&self) -> Iter<'_, V> {
        self.raw.iter()
    }

    /// Returns an iterator over all key-value pairs.
    ///
    /// It acts like `self.iter().enumerate()`,
    /// but use `K` instead of `usize` for iteration indices.
    ///
    /// See [`slice::iter`] for more details.
    ///
    /// # Example
    ///
    /// ```
    /// # use derive_more::{From, Into};
    /// # use typed_index_collections::TiSlice;
    /// #[derive(Debug, Eq, From, Into, PartialEq)]
    /// pub struct Id(usize);
    /// let slice: &TiSlice<Id, usize> = TiSlice::from_ref(&[1, 2, 4]);
    /// let mut iterator = slice.iter_enumerated();
    /// assert_eq!(iterator.next(), Some((Id(0), &1)));
    /// assert_eq!(iterator.next(), Some((Id(1), &2)));
    /// assert_eq!(iterator.next(), Some((Id(2), &4)));
    /// assert_eq!(iterator.next(), None);
    /// ```
    ///
    /// [`slice::iter`]: https://doc.rust-lang.org/std/primitive.slice.html#method.iter
    #[inline]
    pub fn iter_enumerated(&self) -> TiEnumerated<Iter<'_, V>, K, &V>
    where
        K: From<usize>,
    {
        self.raw
            .iter()
            .enumerate()
            .map(|(key, value)| (key.into(), value))
    }

    /// Returns an iterator that allows modifying each value.
    ///
    /// See [`slice::iter_mut`] for more details.
    ///
    /// [`slice::iter_mut`]: https://doc.rust-lang.org/std/primitive.slice.html#method.iter_mut
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, V> {
        self.raw.iter_mut()
    }

    /// Returns an iterator over all key-value pairs, with mutable references to the values.
    ///
    /// It acts like `self.iter_mut().enumerate()`,
    /// but use `K` instead of `usize` for iteration indices.
    ///
    /// # Example
    ///
    /// ```
    /// # use derive_more::{From, Into};
    /// # use typed_index_collections::TiSlice;
    /// #[derive(Debug, Eq, From, Into, PartialEq)]
    /// pub struct Id(usize);
    /// let mut array = [1, 2, 4];
    /// let slice: &mut TiSlice<Id, usize> = TiSlice::from_mut(&mut array);
    /// for (key, value) in slice.iter_mut_enumerated() {
    ///     *value += key.0;
    /// }
    /// assert_eq!(array, [1, 3, 6]);
    /// ```
    ///
    /// [`slice::iter_mut`]: https://doc.rust-lang.org/std/primitive.slice.html#method.iter_mut
    #[inline]
    pub fn iter_mut_enumerated(&mut self) -> TiEnumerated<IterMut<'_, V>, K, &mut V>
    where
        K: From<usize>,
    {
        self.raw
            .iter_mut()
            .enumerate()
            .map(|(key, value)| (key.into(), value))
    }

    /// Searches for an element in an iterator, returning its index of type `K`.
    ///
    /// It acts like `self.iter().position(...)`,
    /// but instead of `usize` it returns index of type `K`.
    ///
    /// See [`slice::iter`] and [`Iterator::position`] for more details.
    ///
    /// # Example
    ///
    /// ```
    /// # use derive_more::{From, Into};
    /// # use typed_index_collections::TiSlice;
    /// #[derive(Debug, Eq, From, Into, PartialEq)]
    /// pub struct Id(usize);
    /// let slice: &TiSlice<Id, usize> = TiSlice::from_ref(&[1, 2, 4, 2, 1]);
    /// assert_eq!(slice.position(|&value| value == 1), Some(Id(0)));
    /// assert_eq!(slice.position(|&value| value == 2), Some(Id(1)));
    /// assert_eq!(slice.position(|&value| value == 3), None);
    /// assert_eq!(slice.position(|&value| value == 4), Some(Id(2)));
    /// ```
    ///
    /// [`slice::iter`]: https://doc.rust-lang.org/std/primitive.slice.html#method.iter
    /// [`Iterator::position`]: https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.position
    #[inline]
    pub fn position<P>(&self, predicate: P) -> Option<K>
    where
        K: From<usize>,
        P: FnMut(&V) -> bool,
    {
        self.raw.iter().position(predicate).map(Into::into)
    }

    /// Searches for an element in an iterator from the right, returning its index of type `K`.
    ///
    /// It acts like `self.iter().rposition(...)`,
    /// but instead of `usize` it returns index of type `K`.
    ///
    /// See [`slice::iter`] and [`Iterator::position`] for more details.
    ///
    /// # Example
    ///
    /// ```
    /// # use derive_more::{From, Into};
    /// # use typed_index_collections::TiSlice;
    /// #[derive(Debug, Eq, From, Into, PartialEq)]
    /// pub struct Id(usize);
    /// let slice: &TiSlice<Id, usize> = TiSlice::from_ref(&[1, 2, 4, 2, 1]);
    /// assert_eq!(slice.rposition(|&value| value == 1), Some(Id(4)));
    /// assert_eq!(slice.rposition(|&value| value == 2), Some(Id(3)));
    /// assert_eq!(slice.rposition(|&value| value == 3), None);
    /// assert_eq!(slice.rposition(|&value| value == 4), Some(Id(2)));
    /// ```
    ///
    /// [`slice::iter`]: https://doc.rust-lang.org/std/primitive.slice.html#method.iter
    /// [`Iterator::position`]: https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.position
    #[inline]
    pub fn rposition<P>(&self, predicate: P) -> Option<K>
    where
        K: From<usize>,
        P: FnMut(&V) -> bool,
    {
        self.raw.iter().rposition(predicate).map(Into::into)
    }

    /// Returns an iterator over all contiguous windows of length
    /// `size`. The windows overlap. If the slice is shorter than
    /// `size`, the iterator returns no values.
    ///
    /// See [`slice::windows`] for more details.
    ///
    /// [`slice::windows`]: https://doc.rust-lang.org/std/primitive.slice.html#method.windows
    #[inline]
    pub fn windows(&self, size: usize) -> TiSliceRefMap<Windows<'_, V>, K, V> {
        self.raw.windows(size).map(TiSlice::from_ref)
    }

    /// Returns an iterator over `chunk_size` elements of the slice at a time, starting at the
    /// beginning of the slice.
    ///
    /// See [`slice::chunks`] for more details.
    ///
    /// [`slice::chunks`]: https://doc.rust-lang.org/std/primitive.slice.html#method.chunks
    #[inline]
    pub fn chunks(&self, chunk_size: usize) -> TiSliceRefMap<Chunks<'_, V>, K, V> {
        self.raw.chunks(chunk_size).map(TiSlice::from_ref)
    }

    /// Returns an iterator over `chunk_size` elements of the slice at a time, starting at the
    /// beginning of the slice.
    ///
    /// See [`slice::chunks_mut`] for more details.
    ///
    /// [`slice::chunks_mut`]: https://doc.rust-lang.org/std/primitive.slice.html#method.chunks_mut
    #[inline]
    pub fn chunks_mut(&mut self, chunk_size: usize) -> TiSliceMutMap<ChunksMut<'_, V>, K, V> {
        self.raw.chunks_mut(chunk_size).map(TiSlice::from_mut)
    }

    /// Returns an iterator over `chunk_size` elements of the slice at a time, starting at the
    /// beginning of the slice.
    ///
    /// See [`slice::chunks_exact`] for more details.
    ///
    /// [`slice::chunks_exact`]: https://doc.rust-lang.org/std/primitive.slice.html#method.chunks_exact
    #[inline]
    pub fn chunks_exact(&self, chunk_size: usize) -> TiSliceRefMap<ChunksExact<'_, V>, K, V> {
        self.raw.chunks_exact(chunk_size).map(TiSlice::from_ref)
    }

    /// Returns an iterator over `chunk_size` elements of the slice at a time, starting at the
    /// beginning of the slice.
    ///
    /// See [`slice::chunks_exact_mut`] for more details.
    ///
    /// [`slice::chunks_exact_mut`]: https://doc.rust-lang.org/std/primitive.slice.html#method.chunks_exact_mut
    #[inline]
    pub fn chunks_exact_mut(
        &mut self,
        chunk_size: usize,
    ) -> TiSliceMutMap<ChunksExactMut<'_, V>, K, V> {
        self.raw.chunks_exact_mut(chunk_size).map(TiSlice::from_mut)
    }

    /// Returns an iterator over `chunk_size` elements of the slice at a time, starting at the end
    /// of the slice.
    ///
    /// See [`slice::rchunks`] for more details.
    ///
    /// [`slice::rchunks`]: https://doc.rust-lang.org/std/primitive.slice.html#method.rchunks
    #[inline]
    pub fn rchunks(&self, chunk_size: usize) -> TiSliceRefMap<RChunks<'_, V>, K, V> {
        self.raw.rchunks(chunk_size).map(TiSlice::from_ref)
    }

    /// Returns an iterator over `chunk_size` elements of the slice at a time, starting at the end
    /// of the slice.
    ///
    /// See [`slice::rchunks_mut`] for more details.
    ///
    /// [`slice::rchunks_mut`]: https://doc.rust-lang.org/std/primitive.slice.html#method.rchunks_mut
    #[inline]
    pub fn rchunks_mut(&mut self, chunk_size: usize) -> TiSliceMutMap<RChunksMut<'_, V>, K, V> {
        self.raw.rchunks_mut(chunk_size).map(TiSlice::from_mut)
    }

    /// Returns an iterator over `chunk_size` elements of the slice at a time, starting at the
    /// end of the slice.
    ///
    /// See [`slice::rchunks_exact`] for more details.
    ///
    /// [`slice::rchunks_exact`]: https://doc.rust-lang.org/std/primitive.slice.html#method.rchunks_exact
    #[inline]
    pub fn rchunks_exact(&self, chunk_size: usize) -> TiSliceRefMap<RChunksExact<'_, V>, K, V> {
        self.raw.rchunks_exact(chunk_size).map(TiSlice::from_ref)
    }

    /// Returns an iterator over `chunk_size` elements of the slice at a time, starting at the end
    /// of the slice.
    ///
    /// See [`slice::rchunks_exact_mut`] for more details.
    ///
    /// [`slice::rchunks_exact_mut`]: https://doc.rust-lang.org/std/primitive.slice.html#method.rchunks_exact_mut
    #[inline]
    pub fn rchunks_exact_mut(
        &mut self,
        chunk_size: usize,
    ) -> TiSliceMutMap<RChunksExactMut<'_, V>, K, V> {
        self.raw
            .rchunks_exact_mut(chunk_size)
            .map(TiSlice::from_mut)
    }

    /// Divides one slice into two at an index.
    ///
    /// See [`slice::split_at`] for more details.
    ///
    /// [`slice::split_at`]: https://doc.rust-lang.org/std/primitive.slice.html#method.split_at
    #[inline]
    pub fn split_at(&self, mid: K) -> (&Self, &Self)
    where
        usize: From<K>,
    {
        let (left, right) = self.raw.split_at(mid.into());
        (left.as_ref(), right.as_ref())
    }

    /// Divides one mutable slice into two at an index.
    ///
    /// See [`slice::split_at_mut`] for more details.
    ///
    /// [`slice::split_at_mut`]: https://doc.rust-lang.org/std/primitive.slice.html#method.split_at_mut
    #[inline]
    pub fn split_at_mut(&mut self, mid: K) -> (&mut Self, &mut Self)
    where
        usize: From<K>,
    {
        let (left, right) = self.raw.split_at_mut(mid.into());
        (left.as_mut(), right.as_mut())
    }

    /// Returns an iterator over subslices separated by elements that match
    /// `pred`. The matched element is not contained in the subslices.
    ///
    /// See [`slice::split`] for more details.
    ///
    /// [`slice::split`]: https://doc.rust-lang.org/std/primitive.slice.html#method.split
    #[inline]
    pub fn split<F>(&self, pred: F) -> TiSliceRefMap<Split<'_, V, F>, K, V>
    where
        F: FnMut(&V) -> bool,
    {
        self.raw.split(pred).map(TiSlice::from_ref)
    }

    /// Returns an iterator over mutable subslices separated by elements that
    /// match `pred`. The matched element is not contained in the subslices.
    ///
    /// See [`slice::split_mut`] for more details.
    ///
    /// [`slice::split_mut`]: https://doc.rust-lang.org/std/primitive.slice.html#method.split_mut
    #[inline]
    pub fn split_mut<F>(&mut self, pred: F) -> TiSliceMutMap<SplitMut<'_, V, F>, K, V>
    where
        F: FnMut(&V) -> bool,
    {
        self.raw.split_mut(pred).map(TiSlice::from_mut)
    }

    /// Returns an iterator over subslices separated by elements that match
    /// `pred`, starting at the end of the slice and working backwards.
    /// The matched element is not contained in the subslices.
    ///
    /// See [`slice::rsplit`] for more details.
    ///
    /// [`slice::rsplit`]: https://doc.rust-lang.org/std/primitive.slice.html#method.rsplit
    #[inline]
    pub fn rsplit<F>(&self, pred: F) -> TiSliceRefMap<RSplit<'_, V, F>, K, V>
    where
        F: FnMut(&V) -> bool,
    {
        self.raw.rsplit(pred).map(TiSlice::from_ref)
    }

    /// Returns an iterator over mutable subslices separated by elements that
    /// match `pred`, starting at the end of the slice and working
    /// backwards. The matched element is not contained in the subslices.
    ///
    /// See [`slice::rsplit_mut`] for more details.
    ///
    /// [`slice::rsplit_mut`]: https://doc.rust-lang.org/std/primitive.slice.html#method.rsplit_mut
    #[inline]
    pub fn rsplit_mut<F>(&mut self, pred: F) -> TiSliceMutMap<RSplitMut<'_, V, F>, K, V>
    where
        F: FnMut(&V) -> bool,
    {
        self.raw.rsplit_mut(pred).map(TiSlice::from_mut)
    }

    /// Returns an iterator over subslices separated by elements that match
    /// `pred`, limited to returning at most `n` items. The matched element is
    /// not contained in the subslices.
    ///
    /// See [`slice::splitn`] for more details.
    ///
    /// [`slice::splitn`]: https://doc.rust-lang.org/std/primitive.slice.html#method.splitn
    #[inline]
    pub fn splitn<F>(&self, n: usize, pred: F) -> TiSliceRefMap<SplitN<'_, V, F>, K, V>
    where
        F: FnMut(&V) -> bool,
    {
        self.raw.splitn(n, pred).map(TiSlice::from_ref)
    }

    /// Returns an iterator over subslices separated by elements that match
    /// `pred`, limited to returning at most `n` items. The matched element is
    /// not contained in the subslices.
    ///
    /// See [`slice::splitn_mut`] for more details.
    ///
    /// [`slice::splitn_mut`]: https://doc.rust-lang.org/std/primitive.slice.html#method.splitn_mut
    #[inline]
    pub fn splitn_mut<F>(&mut self, n: usize, pred: F) -> TiSliceMutMap<SplitNMut<'_, V, F>, K, V>
    where
        F: FnMut(&V) -> bool,
    {
        self.raw.splitn_mut(n, pred).map(TiSlice::from_mut)
    }

    /// Returns an iterator over subslices separated by elements that match
    /// `pred` limited to returning at most `n` items. This starts at the end of
    /// the slice and works backwards. The matched element is not contained in
    /// the subslices.
    ///
    /// See [`slice::rsplitn`] for more details.
    ///
    /// [`slice::rsplitn`]: https://doc.rust-lang.org/std/primitive.slice.html#method.rsplitn
    #[inline]
    pub fn rsplitn<F>(&self, n: usize, pred: F) -> TiSliceRefMap<RSplitN<'_, V, F>, K, V>
    where
        F: FnMut(&V) -> bool,
    {
        self.raw.rsplitn(n, pred).map(TiSlice::from_ref)
    }

    /// Returns an iterator over subslices separated by elements that match
    /// `pred` limited to returning at most `n` items. This starts at the end of
    /// the slice and works backwards. The matched element is not contained in
    /// the subslices.
    ///
    /// See [`slice::rsplitn_mut`] for more details.
    ///
    /// [`slice::rsplitn_mut`]: https://doc.rust-lang.org/std/primitive.slice.html#method.rsplitn_mut
    #[inline]
    pub fn rsplitn_mut<F>(&mut self, n: usize, pred: F) -> TiSliceMutMap<RSplitNMut<'_, V, F>, K, V>
    where
        F: FnMut(&V) -> bool,
    {
        self.raw.rsplitn_mut(n, pred).map(TiSlice::from_mut)
    }

    /// Returns `true` if the slice contains an element with the given value.
    ///
    /// See [`slice::contains`] for more details.
    ///
    /// [`slice::contains`]: https://doc.rust-lang.org/std/primitive.slice.html#method.contains
    pub fn contains(&self, x: &V) -> bool
    where
        V: PartialEq,
    {
        self.raw.contains(x)
    }

    /// Returns `true` if `needle` is a prefix of the slice.
    ///
    /// See [`slice::starts_with`] for more details.
    ///
    /// [`slice::starts_with`]: https://doc.rust-lang.org/std/primitive.slice.html#method.starts_with
    pub fn starts_with(&self, needle: &Self) -> bool
    where
        V: PartialEq,
    {
        self.raw.starts_with(needle.as_ref())
    }

    /// Returns `true` if `needle` is a suffix of the slice.
    ///
    /// See [`slice::ends_with`] for more details.
    ///
    /// [`slice::ends_with`]: https://doc.rust-lang.org/std/primitive.slice.html#method.ends_with
    pub fn ends_with(&self, needle: &Self) -> bool
    where
        V: PartialEq,
    {
        self.raw.ends_with(needle.as_ref())
    }

    /// Binary searches this sorted slice for a given element.
    ///
    /// See [`slice::binary_search`] for more details.
    ///
    /// [`slice::binary_search`]: https://doc.rust-lang.org/std/primitive.slice.html#method.binary_search
    pub fn binary_search(&self, x: &V) -> Result<K, K>
    where
        V: Ord,
        K: From<usize>,
    {
        self.raw
            .binary_search(x)
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Binary searches this sorted slice with a comparator function.
    ///
    /// See [`slice::binary_search_by`] for more details.
    ///
    /// [`slice::binary_search_by`]: https://doc.rust-lang.org/std/primitive.slice.html#method.binary_search_by
    #[inline]
    pub fn binary_search_by<'a, F>(&'a self, f: F) -> Result<K, K>
    where
        F: FnMut(&'a V) -> Ordering,
        K: From<usize>,
    {
        self.raw
            .binary_search_by(f)
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Binary searches this sorted slice with a key extraction function.
    ///
    /// See [`slice::binary_search_by_key`] for more details.
    ///
    /// [`slice::binary_search_by_key`]: https://doc.rust-lang.org/std/primitive.slice.html#method.binary_search_by_key
    #[inline]
    pub fn binary_search_by_key<'a, B, F>(&'a self, b: &B, f: F) -> Result<K, K>
    where
        F: FnMut(&'a V) -> B,
        B: Ord,
        K: From<usize>,
    {
        self.raw
            .binary_search_by_key(b, f)
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Sorts the slice, but may not preserve the order of equal elements.
    ///
    /// See [`slice::sort_unstable`] for more details.
    ///
    /// [`slice::sort_unstable`]: https://doc.rust-lang.org/std/primitive.slice.html#method.sort_unstable
    #[inline]
    pub fn sort_unstable(&mut self)
    where
        V: Ord,
    {
        self.raw.sort_unstable()
    }

    /// Sorts the slice with a comparator function, but may not preserve the order of equal
    /// elements.
    ///
    /// See [`slice::sort_unstable_by`] for more details.
    ///
    /// [`slice::sort_unstable_by`]: https://doc.rust-lang.org/std/primitive.slice.html#method.sort_unstable_by
    #[inline]
    pub fn sort_unstable_by<F>(&mut self, compare: F)
    where
        F: FnMut(&V, &V) -> Ordering,
    {
        self.raw.sort_unstable_by(compare)
    }

    /// Sorts the slice with a key extraction function, but may not preserve the order of equal
    /// elements.
    ///
    /// See [`slice::sort_unstable_by_key`] for more details.
    ///
    /// [`slice::sort_unstable_by_key`]: https://doc.rust-lang.org/std/primitive.slice.html#method.sort_unstable_by_key
    #[inline]
    pub fn sort_unstable_by_key<K2, F>(&mut self, f: F)
    where
        F: FnMut(&V) -> K2,
        K2: Ord,
    {
        self.raw.sort_unstable_by_key(f)
    }

    /// Rotates the slice in-place such that the first `mid` elements of the
    /// slice move to the end while the last `self.next_key() - mid` elements move to
    /// the front. After calling `rotate_left`, the element previously at index
    /// `mid` will become the first element in the slice.
    ///
    /// See [`slice::rotate_left`] for more details.
    ///
    /// [`slice::rotate_left`]: https://doc.rust-lang.org/std/primitive.slice.html#method.rotate_left
    pub fn rotate_left(&mut self, mid: K)
    where
        usize: From<K>,
    {
        self.raw.rotate_left(mid.into())
    }

    /// Rotates the slice in-place such that the first `self.next_key() - k`
    /// elements of the slice move to the end while the last `k` elements move
    /// to the front. After calling `rotate_right`, the element previously at
    /// index `self.next_key() - k` will become the first element in the slice.
    ///
    /// See [`slice::rotate_right`] for more details.
    ///
    /// [`slice::rotate_right`]: https://doc.rust-lang.org/std/primitive.slice.html#method.rotate_right
    pub fn rotate_right(&mut self, k: K)
    where
        usize: From<K>,
    {
        self.raw.rotate_right(k.into())
    }

    /// Copies the elements from `src` into `self`.
    ///
    /// See [`slice::clone_from_slice`] for more details.
    ///
    /// [`slice::clone_from_slice`]: https://doc.rust-lang.org/std/primitive.slice.html#method.clone_from_slice
    pub fn clone_from_slice(&mut self, src: &Self)
    where
        V: Clone,
    {
        self.raw.clone_from_slice(&src.raw)
    }

    /// Copies all elements from `src` into `self`, using a memcpy.
    ///
    /// See [`slice::copy_from_slice`] for more details.
    ///
    /// [`slice::copy_from_slice`]: https://doc.rust-lang.org/std/primitive.slice.html#method.copy_from_slice
    pub fn copy_from_slice(&mut self, src: &Self)
    where
        V: Copy,
    {
        self.raw.copy_from_slice(&src.raw)
    }

    /// Copies elements from one part of the slice to another part of itself,
    /// using a memmove.
    ///
    /// See [`slice::copy_within`] for more details.
    ///
    /// [`slice::copy_within`]: https://doc.rust-lang.org/std/primitive.slice.html#method.copy_within
    pub fn copy_within<R>(&mut self, src: R, dest: K)
    where
        R: TiRangeBounds<K>,
        V: Copy,
        usize: From<K>,
    {
        self.raw.copy_within(src.into_range(), dest.into())
    }

    /// Swaps all elements in `self` with those in `other`.
    ///
    ///
    /// See [`slice::swap_with_slice`] for more details.
    ///
    /// [`slice::swap_with_slice`]: https://doc.rust-lang.org/std/primitive.slice.html#method.swap_with_slice
    pub fn swap_with_slice(&mut self, other: &mut Self) {
        self.raw.swap_with_slice(other.as_mut())
    }

    /// Transmute the slice to a slice of another type, ensuring alignment of the types is
    /// maintained.
    ///
    /// See [`slice::align_to`] for more details.
    ///
    /// [`slice::align_to`]: https://doc.rust-lang.org/std/primitive.slice.html#method.align_to
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn align_to<U>(&self) -> (&Self, &TiSlice<K, U>, &Self) {
        let (first, mid, last) = self.raw.align_to();
        (first.as_ref(), mid.as_ref(), last.as_ref())
    }

    /// Transmute the slice to a slice of another type, ensuring alignment of the types is
    /// maintained.
    ///
    /// See [`slice::align_to_mut`] for more details.
    ///
    /// [`slice::align_to_mut`]: https://doc.rust-lang.org/std/primitive.slice.html#method.align_to_mut
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn align_to_mut<U>(&mut self) -> (&mut Self, &mut TiSlice<K, U>, &mut Self) {
        let (first, mid, last) = self.raw.align_to_mut();
        (first.as_mut(), mid.as_mut(), last.as_mut())
    }
}

impl<K> TiSlice<K, u8> {
    /// Checks if all bytes in this slice are within the ASCII range.
    ///
    /// See [`slice::is_ascii`] for more details.
    ///
    /// [`slice::is_ascii`]: https://doc.rust-lang.org/std/primitive.slice.html#method.is_ascii
    #[inline]
    pub fn is_ascii(&self) -> bool {
        self.raw.is_ascii()
    }

    /// Checks that two slices are an ASCII case-insensitive match.
    ///
    /// See [`slice::eq_ignore_ascii_case`] for more details.
    ///
    /// [`slice::eq_ignore_ascii_case`]: https://doc.rust-lang.org/std/primitive.slice.html#method.eq_ignore_ascii_case
    #[inline]
    pub fn eq_ignore_ascii_case(&self, other: &Self) -> bool {
        self.raw.eq_ignore_ascii_case(other.as_ref())
    }

    /// Converts this slice to its ASCII upper case equivalent in-place.
    ///
    /// See [`slice::make_ascii_uppercase`] for more details.
    ///
    /// [`slice::make_ascii_uppercase`]: https://doc.rust-lang.org/std/primitive.slice.html#method.make_ascii_uppercase
    #[inline]
    pub fn make_ascii_uppercase(&mut self) {
        self.raw.make_ascii_uppercase()
    }

    /// Converts this slice to its ASCII lower case equivalent in-place.
    ///
    /// See [`slice::make_ascii_lowercase`] for more details.
    ///
    /// [`slice::make_ascii_lowercase`]: https://doc.rust-lang.org/std/primitive.slice.html#method.make_ascii_lowercase
    #[inline]
    pub fn make_ascii_lowercase(&mut self) {
        self.raw.make_ascii_lowercase()
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl<K, V> TiSlice<K, V> {
    /// Sorts the slice.
    ///
    /// See [`slice::sort`] for more details.
    ///
    /// [`slice::sort`]: https://doc.rust-lang.org/std/primitive.slice.html#method.sort
    #[inline]
    pub fn sort(&mut self)
    where
        V: Ord,
    {
        self.raw.sort()
    }

    /// Sorts the slice with a comparator function.
    ///
    /// See [`slice::sort_by`] for more details.
    ///
    /// [`slice::sort_by`]: https://doc.rust-lang.org/std/primitive.slice.html#method.sort_by
    #[inline]
    pub fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&V, &V) -> Ordering,
    {
        self.raw.sort_by(compare)
    }

    /// Sorts the slice with a key extraction function.
    ///
    /// See [`slice::sort_by_key`] for more details.
    ///
    /// [`slice::sort_by_key`]: https://doc.rust-lang.org/std/primitive.slice.html#method.sort_by_key
    #[inline]
    pub fn sort_by_key<K2, F>(&mut self, f: F)
    where
        F: FnMut(&V) -> K2,
        K2: Ord,
    {
        self.raw.sort_by_key(f)
    }

    /// Sorts the slice with a key extraction function.
    ///
    /// See [`slice::sort_by_cached_key`] for more details.
    ///
    /// [`slice::sort_by_cached_key`]: https://doc.rust-lang.org/std/primitive.slice.html#method.sort_by_cached_key
    #[inline]
    pub fn sort_by_cached_key<K2, F>(&mut self, f: F)
    where
        F: FnMut(&V) -> K2,
        K2: Ord,
    {
        self.raw.sort_by_cached_key(f)
    }

    /// Copies `self` into a new `TiVec`.
    ///
    /// See [`slice::to_vec`] for more details.
    ///
    /// [`slice::to_vec`]: https://doc.rust-lang.org/std/primitive.slice.html#method.to_vec
    #[inline]
    pub fn to_vec(&self) -> TiVec<K, V>
    where
        V: Clone,
    {
        self.raw.to_vec().into()
    }

    /// Converts `self` into a vector without clones or allocation.
    ///
    /// See [`slice::into_vec`] for more details.
    ///
    /// [`slice::into_vec`]: https://doc.rust-lang.org/std/primitive.slice.html#method.into_vec
    #[inline]
    pub fn into_vec(self: Box<Self>) -> TiVec<K, V> {
        Box::<[V]>::from(self).into_vec().into()
    }

    /// Creates a vector by repeating a slice `n` times.
    ///
    /// See [`slice::repeat`] for more details.
    ///
    /// [`slice::repeat`]: https://doc.rust-lang.org/std/primitive.slice.html#method.repeat
    pub fn repeat(&self, n: usize) -> TiVec<K, V>
    where
        V: Copy,
    {
        self.raw.repeat(n).into()
    }

    /// Flattens a slice of `T` into a single value `Self::Output`.
    ///
    /// See [`slice::concat`] for more details.
    ///
    /// [`slice::concat`]: https://doc.rust-lang.org/std/primitive.slice.html#method.concat
    pub fn concat<Item: ?Sized>(&self) -> <Self as Concat<Item>>::Output
    where
        Self: Concat<Item>,
    {
        Concat::concat(self)
    }

    /// Flattens a slice of `T` into a single value `Self::Output`, placing a
    /// given separator between each.
    ///
    /// See [`slice::join`] for more details.
    ///
    /// [`slice::join`]: https://doc.rust-lang.org/std/primitive.slice.html#method.join
    pub fn join<Separator>(&self, sep: Separator) -> <Self as Join<Separator>>::Output
    where
        Self: Join<Separator>,
    {
        Join::join(self, sep)
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl<K> TiSlice<K, u8> {
    /// Returns a vector containing a copy of this slice where each byte
    /// is mapped to its ASCII upper case equivalent.
    ///
    /// See [`slice::to_ascii_uppercase`] for more details.
    ///
    /// [`slice::to_ascii_uppercase`]: https://doc.rust-lang.org/std/primitive.slice.html#method.to_ascii_uppercase
    #[inline]
    pub fn to_ascii_uppercase(&self) -> TiVec<K, u8> {
        self.raw.to_ascii_uppercase().into()
    }

    /// Returns a vector containing a copy of this slice where each byte
    /// is mapped to its ASCII lower case equivalent.
    ///
    /// See [`slice::to_ascii_lowercase`] for more details.
    ///
    /// [`slice::to_ascii_lowercase`]: https://doc.rust-lang.org/std/primitive.slice.html#method.to_ascii_lowercase
    #[inline]
    pub fn to_ascii_lowercase(&self) -> TiVec<K, u8> {
        self.raw.to_ascii_lowercase().into()
    }
}

impl<K, V> fmt::Debug for TiSlice<K, V>
where
    K: fmt::Debug + From<usize>,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter_enumerated()).finish()
    }
}

impl<K, V> AsRef<TiSlice<K, V>> for TiSlice<K, V> {
    fn as_ref(&self) -> &TiSlice<K, V> {
        self
    }
}

impl<K, V> AsMut<TiSlice<K, V>> for TiSlice<K, V> {
    fn as_mut(&mut self) -> &mut TiSlice<K, V> {
        self
    }
}

impl<K, V> AsRef<[V]> for TiSlice<K, V> {
    fn as_ref(&self) -> &[V] {
        &self.raw
    }
}

impl<K, V> AsMut<[V]> for TiSlice<K, V> {
    fn as_mut(&mut self) -> &mut [V] {
        &mut self.raw
    }
}

impl<K, V> AsRef<TiSlice<K, V>> for [V] {
    fn as_ref(&self) -> &TiSlice<K, V> {
        TiSlice::from_ref(self)
    }
}

impl<K, V> AsMut<TiSlice<K, V>> for [V] {
    fn as_mut(&mut self) -> &mut TiSlice<K, V> {
        TiSlice::from_mut(self)
    }
}

impl<I, K, V> ops::Index<I> for TiSlice<K, V>
where
    I: TiSliceIndex<K, V>,
{
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        index.index(self)
    }
}

impl<I, K, V> ops::IndexMut<I> for TiSlice<K, V>
where
    I: TiSliceIndex<K, V>,
{
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        index.index_mut(self)
    }
}

impl<'a, K, V> IntoIterator for &'a TiSlice<K, V> {
    type Item = &'a V;
    type IntoIter = Iter<'a, V>;

    #[allow(clippy::into_iter_on_ref)]
    fn into_iter(self) -> Iter<'a, V> {
        self.raw.into_iter()
    }
}

impl<'a, K, V> IntoIterator for &'a mut TiSlice<K, V> {
    type Item = &'a mut V;
    type IntoIter = IterMut<'a, V>;

    #[allow(clippy::into_iter_on_ref)]
    fn into_iter(self) -> IterMut<'a, V> {
        (&mut self.raw).into_iter()
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl<K, V: Clone> ToOwned for TiSlice<K, V> {
    type Owned = TiVec<K, V>;
    fn to_owned(&self) -> TiVec<K, V> {
        self.raw.to_owned().into()
    }
}

impl<K, A, B> PartialEq<TiSlice<K, B>> for TiSlice<K, A>
where
    A: PartialEq<B>,
{
    fn eq(&self, other: &TiSlice<K, B>) -> bool {
        self.raw == other.raw
    }
}

impl<K, V> PartialOrd<TiSlice<K, V>> for TiSlice<K, V>
where
    V: PartialOrd<V>,
{
    #[inline]
    fn partial_cmp(&self, other: &TiSlice<K, V>) -> Option<Ordering> {
        self.raw.partial_cmp(&other.raw)
    }
}

impl<K, V> Default for &TiSlice<K, V> {
    #[inline]
    fn default() -> Self {
        TiSlice::from_ref(&[])
    }
}

impl<K, V> Default for &mut TiSlice<K, V> {
    #[inline]
    fn default() -> Self {
        TiSlice::from_mut(&mut [])
    }
}

impl<K, V> Eq for TiSlice<K, V> where V: Eq {}

impl<K, V> Hash for TiSlice<K, V>
where
    V: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.raw.hash(state)
    }
}

impl<K, V> Ord for TiSlice<K, V>
where
    V: Ord,
{
    #[inline]
    fn cmp(&self, other: &TiSlice<K, V>) -> Ordering {
        self.raw.cmp(&other.raw)
    }
}

#[cfg(feature = "serde")]
impl<K, V: Serialize> Serialize for TiSlice<K, V> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.raw.serialize(serializer)
    }
}

#[cfg(test)]
mod test {
    use crate::{test::Id, TiSlice};

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct NonCopy<T>(pub T);

    // Function used to avoid allocations for no_std and no_alloc targets
    #[rustfmt::skip]
    fn array_32_from<T, F>(mut func: F) -> [T; 32]
    where
        F: FnMut() -> T,
    {
        [
            func(), func(), func(), func(), func(), func(), func(), func(),
            func(), func(), func(), func(), func(), func(), func(), func(),
            func(), func(), func(), func(), func(), func(), func(), func(),
            func(), func(), func(), func(), func(), func(), func(), func(),
        ]
    }

    #[test]
    fn no_std_api_compatibility() {
        for_in!(for arr in [[0; 0], [1], [1, 2], [1, 2, 4], [1, 2, 4, 8]] {
            assert_eq_api!(arr => |arr| arr.as_ref().into_t().len());
            assert_eq_api!(arr => |arr| arr.as_ref().into_t().is_empty());
            assert_eq_api!(arr => |&arr| arr.as_ref().into_t().first());
            assert_eq_api!(arr => |&mut arr| arr.as_mut().into_t().first_mut());
            assert_eq_api!(arr => |&arr| arr.as_ref().into_t().split_first().into_t());
            assert_eq_api!(arr => |&mut arr| arr.as_mut().into_t().split_first_mut().into_t());
            assert_eq_api!(arr => |&arr| arr.as_ref().into_t().split_last().into_t());
            assert_eq_api!(arr => |&mut arr| arr.as_mut().into_t().split_last_mut().into_t());
            assert_eq_api!(arr => |&arr| arr.as_ref().into_t().last());
            assert_eq_api!(arr => |&mut arr| arr.as_mut().into_t().last_mut());
            for index in 0..5_usize {
                assert_eq_api!(arr => |&arr| arr.as_ref().into_t().get(index.into_t()));
                assert_eq_api!(arr => |&mut arr| arr.as_mut().into_t().get_mut(index.into_t()));
            }
            for index in 0..arr.len() {
                unsafe {
                    assert_eq_api!(arr => |&arr| arr.as_ref().into_t()
                        .get_unchecked(index.into_t()));
                    assert_eq_api!(arr => |&mut arr| arr.as_mut().into_t()
                        .get_unchecked_mut(index.into_t()));
                }
            }
            for start in 0..5usize {
                for end in 0..5usize {
                    assert_eq_api!(arr => |&arr| arr.as_ref().into_t()
                        .get(start.into_t()..end.into_t()).into_t());
                    assert_eq_api!(arr => |&mut arr| arr.as_mut().into_t()
                        .get_mut(start.into_t()..end.into_t()).into_t());
                    assert_eq_api!(arr => |&arr| arr.as_ref().into_t()
                        .get(start.into_t()..=end.into_t()).into_t());
                    assert_eq_api!(arr => |&mut arr| arr.as_mut().into_t()
                        .get_mut(start.into_t()..=end.into_t()).into_t());
                }
            }
            unsafe {
                for start in 0..arr.len() {
                    for end in start..arr.len() {
                        assert_eq_api!(arr => |&arr| arr.as_ref().into_t()
                            .get_unchecked(start.into_t()..end.into_t()).into_t());
                        assert_eq_api!(arr => |&mut arr| arr.as_mut().into_t()
                            .get_unchecked_mut(start.into_t()..end.into_t()).into_t());
                    }
                }
                if !arr.is_empty() {
                    for start in 0..arr.len() - 1 {
                        for end in start..arr.len() - 1 {
                            assert_eq_api!(arr => |&arr| arr.as_ref().into_t()
                                .get_unchecked(start.into_t()..=end.into_t()).into_t());
                            assert_eq_api!(arr => |&mut arr| arr.as_mut().into_t()
                                .get_unchecked_mut(start.into_t()..=end.into_t()).into_t());
                        }
                    }
                }
            }
            for start in 0..5usize {
                assert_eq_api!(arr => |&arr| arr.as_ref().into_t()
                    .get(start.into_t()..).into_t());
                assert_eq_api!(arr => |&mut arr| arr.as_mut().into_t()
                    .get_mut(start.into_t()..).into_t());
            }
            unsafe {
                for start in 0..arr.len() {
                    assert_eq_api!(arr => |&arr| arr.as_ref().into_t()
                        .get_unchecked(start.into_t()..).into_t());
                    assert_eq_api!(arr => |&mut arr| arr.as_mut().into_t()
                        .get_unchecked_mut(start.into_t()..).into_t());
                }
            }
            for end in 0..5usize {
                assert_eq_api!(arr => |&arr| arr.as_ref().into_t()
                    .get(..end.into_t()).into_t());
                assert_eq_api!(arr => |&mut arr| arr.as_mut().into_t()
                    .get_mut(..end.into_t()).into_t());
                assert_eq_api!(arr => |&arr| arr.as_ref().into_t()
                    .get(..=end.into_t()).into_t());
                assert_eq_api!(arr => |&mut arr| arr.as_mut().into_t()
                    .get_mut(..=end.into_t()).into_t());
            }
            unsafe {
                for end in 0..arr.len() {
                    assert_eq_api!(arr => |&arr| arr.as_ref().into_t()
                        .get_unchecked(..end.into_t()).into_t());
                    assert_eq_api!(arr => |&mut arr| arr.as_mut().into_t()
                        .get_unchecked_mut(..end.into_t()).into_t());
                }
                if !arr.is_empty() {
                    for end in 0..arr.len() - 1 {
                        assert_eq_api!(arr => |&arr| arr.as_ref().into_t()
                            .get_unchecked(..=end.into_t()).into_t());
                        assert_eq_api!(arr => |&mut arr| arr.as_mut().into_t()
                            .get_unchecked_mut(..=end.into_t()).into_t());
                    }
                }
            }
        });
        for_in!(
            for arr in [&[0; 0], &[1], &[1, 2], &[1, 2, 4], &[1, 2, 4, 8]] {
                assert_eq_api!(arr => |arr| arr.into_t().as_ptr());
            }
        );
        for_in!(for arr in [[0; 0], [1], [1, 2], [1, 2, 4], [1, 2, 4, 8]] {
            assert_ne_api!(arr => |&mut arr| arr.as_mut().into_t().as_mut_ptr());
        });
        assert_eq_api!([1u32, 2, 3] => |&mut arr| {
            arr.as_mut().into_t().swap(0.into_t(), 2.into_t());
            arr
        });
        for_in!(for arr in [[0; 0], [1], [1, 2], [1, 2, 4], [1, 2, 4, 8]] {
            assert_eq_api!(arr => |&mut arr| {
                arr.as_mut().into_t().reverse();
                arr
            });
            assert_eq_api!(arr => |&arr| {
                let mut iter = arr.as_ref().into_t().iter();
                array_32_from(|| iter.next())
            });
            assert_eq_api!(arr => |&mut arr| {
                arr.as_mut().into_t().iter_mut().for_each(|item| *item += 1);
                arr
            });
            for chunk_size in 1..5 {
                assert_eq_api!(arr => |&arr| {
                    let mut windows = arr.as_ref().into_t().windows(chunk_size);
                    array_32_from(|| windows.next().into_t())
                });
                assert_eq_api!(arr => |&arr| {
                    let mut chunks = arr.as_ref().into_t().chunks(chunk_size);
                    array_32_from(|| chunks.next().into_t())
                });
                assert_eq_api!(arr => |&mut arr| {
                    arr.as_mut().into_t().chunks_mut(chunk_size)
                        .for_each(|slice| slice.iter_mut().for_each(|item| *item += 1));
                    arr
                });
                assert_eq_api!(arr => |&arr| {
                    let mut chunks = arr.as_ref().into_t().chunks_exact(chunk_size);
                    array_32_from(|| chunks.next().into_t())
                });
                assert_eq_api!(arr => |&mut arr| {
                    arr.as_mut().into_t().chunks_exact_mut(chunk_size)
                        .for_each(|slice| slice.iter_mut().for_each(|item| *item += 1));
                    arr
                });
                assert_eq_api!(arr => |&arr| {
                    let mut chunks = arr.as_ref().into_t().rchunks(chunk_size);
                    array_32_from(|| chunks.next().into_t())
                });
                assert_eq_api!(arr => |&mut arr| {
                    arr.as_mut().into_t().rchunks_mut(chunk_size)
                        .for_each(|slice| slice.iter_mut().for_each(|item| *item += 1));
                    arr
                });
                assert_eq_api!(arr => |&arr| {
                    let mut chunks = arr.as_ref().into_t().rchunks_exact(chunk_size);
                    array_32_from(|| chunks.next().into_t())
                });
                assert_eq_api!(arr => |&mut arr| {
                    arr.as_mut().into_t().rchunks_exact_mut(chunk_size)
                        .for_each(|slice| slice.iter_mut().for_each(|item| *item += 1));
                    arr
                });
            }
        });
        for_in!(for arr in [[0; 0], [1], [1, 2], [1, 2, 4], [1, 2, 4, 8]] {
            for mid in 0..arr.len() {
                assert_eq_api!(arr => |&arr| {
                    arr.as_ref().into_t().split_at(mid.into_t()).into_t()
                });
                assert_eq_api!(arr => |&mut arr| {
                    arr.as_mut().into_t().split_at_mut(mid.into_t()).into_t()
                });
            }
            for div in 1..5 {
                assert_eq_api!(arr => |&arr| {
                    let mut split = arr.as_ref().into_t().split(|v| v % div == 0);
                    array_32_from(|| split.next().into_t())
                });
                assert_eq_api!(arr => |&mut arr| {
                    let mut split = arr.as_mut().into_t().split_mut(|v| v % div == 0);
                    array_32_from(|| split.next().into_t())
                });
                assert_eq_api!(arr => |&arr| {
                    let mut split = arr.as_ref().into_t().rsplit(|v| v % div == 0);
                    array_32_from(|| split.next().into_t())
                });
                assert_eq_api!(arr => |&mut arr| {
                    let mut split = arr.as_mut().into_t().rsplit_mut(|v| v % div == 0);
                    array_32_from(|| split.next().into_t())
                });
                for num in 1..5 {
                    assert_eq_api!(arr => |&arr| {
                        let mut split = arr.as_ref().into_t().splitn(num, |v| v % div == 0);
                        array_32_from(|| split.next().into_t())
                    });
                    assert_eq_api!(arr => |&mut arr| {
                        let mut split = arr.as_mut().into_t().splitn_mut(num, |v| v % div == 0);
                        array_32_from(|| split.next().into_t())
                    });
                    assert_eq_api!(arr => |&arr| {
                        let mut split = arr.as_ref().into_t().rsplitn(num, |v| v % div == 0);
                        array_32_from(|| split.next().into_t())
                    });
                    assert_eq_api!(arr => |&mut arr| {
                        let mut split = arr.as_mut().into_t().rsplitn_mut(num, |v| v % div == 0);
                        array_32_from(|| split.next().into_t())
                    });
                }
            }
        });
        for_in!(for arr in [[0; 0], [1], [1, 2], [1, 3, 5], [1, 2, 3, 4]] {
            for value in 0..5 {
                assert_eq_api!(arr => |&arr| arr.as_ref().into_t().contains(&value));
            }
            for needle in &[
                &[][..],
                &[0][..],
                &[1, 2][..],
                &[3, 4][..],
                &[1, 3][..],
                &[3, 5][..],
            ] {
                assert_eq_api!(arr => |&arr| arr.as_ref().into_t().starts_with(needle.into_t()));
                assert_eq_api!(arr => |&arr| arr.as_ref().into_t().ends_with(needle.into_t()));
            }
            for value in 0..5 {
                assert_eq_api!(arr => |&arr| arr.as_ref().into_t().contains(&value));
                assert_eq_api!(arr => |&arr| arr.as_ref().into_t().binary_search(&value).into_t());
                assert_eq_api!(arr => |&arr| arr.as_ref().into_t()
                    .binary_search_by(|item| item.cmp(&value)).into_t());
                assert_eq_api!(arr => |&arr| arr.as_ref().into_t()
                    .binary_search_by_key(&value, |item| 5 - item).into_t());
            }
        });
        for_in!(
            for arr in [[0; 0], [1], [1, 3], [7, 3, 5], [10, 6, 35, 4]] {
                assert_eq_api!(arr => |&mut arr| {
                    arr.as_mut().into_t().sort_unstable();
                    arr
                });
                assert_eq_api!(arr => |&mut arr| {
                    arr.as_mut().into_t()
                        .sort_unstable_by(|lhs, rhs| rhs.partial_cmp(lhs).unwrap());
                    arr
                });
                assert_eq_api!(arr => |&mut arr| {
                    arr.as_mut().into_t()
                        .sort_unstable_by_key(|&value| value * (value % 3));
                    arr
                });
                for mid in 0..arr.len() {
                    assert_eq_api!(arr => |&mut arr| {
                        arr.as_mut().into_t().rotate_left(mid.into_t());
                        arr
                    });
                    assert_eq_api!(arr => |&mut arr| {
                        arr.as_mut().into_t().rotate_right(mid.into_t());
                        arr
                    });
                }
            }
        );
        for_in!(for src in [
            [NonCopy(0); 0],
            [NonCopy(1)],
            [NonCopy(1), NonCopy(3)],
            [NonCopy(7), NonCopy(3), NonCopy(5)],
            [NonCopy(10), NonCopy(6), NonCopy(35), NonCopy(4)]
        ] {
            assert_eq_api!([
                NonCopy(1),
                NonCopy(2),
                NonCopy(3),
                NonCopy(4),
                NonCopy(5),
                NonCopy(6),
            ] => |&mut arr| {
                arr.as_mut().into_t()[..src.len().into_t()]
                    .clone_from_slice(src.as_ref().into_t());
                arr
            });
        });
        for_in!(
            for src in [[0; 0], [1], [1, 3], [7, 3, 5], [10, 6, 35, 4]] {
                assert_eq_api!([1, 2, 3, 4, 5, 6] => |&mut arr| {
                    arr.as_mut().into_t()[..src.len().into_t()]
                        .copy_from_slice(src.as_ref().into_t());
                    arr
                });
            }
        );
        assert_eq_api!([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12] => |&mut arr| {
            arr.as_mut().into_t().copy_within(1.into_t()..4.into_t(), 6.into_t());
            arr
        });
        assert_eq_api!(([1, 2], [3, 4, 5, 6]) => |mut args| {
            args.0.as_mut().into_t().swap_with_slice((&mut args.1[2..]).into_t());
        });
        {
            let mut arr = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17];
            for start in 0..arr.len() {
                for end in start..arr.len() {
                    unsafe {
                        assert_eq_api!(&arr => |&arr| {
                            arr.as_ref()[start..end].into_t().align_to::<u64>().into_t()
                        });
                        assert_eq!(
                            {
                                let (prefix, values, suffixes) = arr.as_mut().align_to::<u64>();
                                (prefix.to_vec(), values.to_vec(), suffixes.to_vec())
                            },
                            {
                                let (prefix, values, suffixes) =
                                    TiSlice::<Id, _>::from_mut(arr.as_mut()).align_to::<u64>();
                                (
                                    prefix.raw.to_vec(),
                                    values.raw.to_vec(),
                                    suffixes.raw.to_vec(),
                                )
                            }
                        );
                    }
                }
            }
        }
    }

    #[cfg(any(feature = "alloc", feature = "std"))]
    #[test]
    fn std_api_compatibility() {
        use alloc::boxed::Box;

        for_in!(
            for arr in [[0; 0], [1], [1, 3], [7, 3, 5], [10, 6, 35, 4]] {
                assert_eq_api!(arr => |&mut arr| {
                    #[allow(clippy::stable_sort_primitive)]
                    arr.as_mut().into_t().sort();
                    arr
                });
                assert_eq_api!(arr => |&mut arr| {
                    arr.as_mut().into_t().sort_by(|lhs, rhs| rhs.partial_cmp(lhs).unwrap());
                    arr
                });
                assert_eq_api!(arr => |&mut arr| {
                    arr.as_mut().into_t().sort_by_key(|&value| value * (value % 3));
                    arr
                });
                assert_eq_api!(arr => |&mut arr| {
                    arr.as_mut().into_t().sort_by_cached_key(|&value| value * (value % 3));
                    arr
                });
                assert_eq_api!(arr => |&arr| arr.into_t().to_vec().as_slice().into_t());
                assert_eq_api!(arr => |&arr| {
                    let boxed = Box::new(arr.into_t().to_vec().into_boxed_slice());
                    boxed.into_vec().as_slice().into_t()
                });
                assert_eq_api!(arr => |&arr| arr.into_t().repeat(5).as_slice().into_t());
                assert_eq_api!([[1, 2], [3, 4]] => |arr| arr.into_t().concat().as_slice().into_t());
                assert_eq_api!([[1, 2], [3, 4]] => |arr| arr.into_t().join(&0).as_slice().into_t());
            }
        );
    }

    #[test]
    fn no_std_u8_api_compatibility() {
        for_in!(
            for arr in [*b"abc", *b"aBc", *b"ABC", *b"abd", *b"\x80\x81"] {
                assert_eq_api!(arr => |&arr| arr.into_t().is_ascii());
                assert_eq_api!(arr => |&arr| arr.into_t().eq_ignore_ascii_case(b"aBc".into_t()));
                assert_eq_api!(arr => |&mut arr| arr.as_mut().into_t().make_ascii_uppercase());
                assert_eq_api!(arr => |&mut arr| arr.as_mut().into_t().make_ascii_lowercase());
            }
        );
    }

    #[cfg(any(feature = "alloc", feature = "std"))]
    #[test]
    fn std_u8_api_compatibility() {
        for_in!(
            for arr in [*b"abc", *b"aBc", *b"ABC", *b"abd", *b"\x80\x81"] {
                assert_eq_api!(arr => |&arr| arr.into_t().to_ascii_uppercase().into_t());
                assert_eq_api!(arr => |&arr| arr.into_t().to_ascii_lowercase().into_t());
            }
        );
    }

    #[test]
    fn no_std_trait_api_compatibility() {
        use core::slice::IterMut;
        for_in!(for arr in [[0; 0], [1], [1, 2], [1, 2, 4], [1, 2, 4, 8]] {
            assert_eq_api!(arr => |&arr| AsRef::<UsizeSlice>::as_ref(arr.as_ref().into_t()).into_t());
            assert_eq_api!(arr => |&mut arr| AsMut::<UsizeSlice>::as_mut(arr.as_mut().into_t()).into_t());
            for index in 0..arr.len() {
                assert_eq_api!(arr => |&arr| &arr.as_ref().into_t()[index.into_t()]);
                assert_eq_api!(arr => |&mut arr| &mut arr.as_mut().into_t()[index.into_t()]);
            }
            for start in 0..arr.len() {
                for end in start..arr.len() {
                    assert_eq_api!(arr => |&arr| &arr.as_ref().into_t()
                            [start.into_t()..end.into_t()].into_t());
                    assert_eq_api!(arr => |&mut arr| &mut arr.as_mut().into_t()
                            [start.into_t()..end.into_t()].into_t());
                }
            }
            if !arr.is_empty() {
                for start in 0..arr.len() - 1 {
                    for end in start..arr.len() - 1 {
                        assert_eq_api!(arr => |&arr| &arr.as_ref()
                            .into_t()[start.into_t()..=end.into_t()].into_t());
                        assert_eq_api!(arr => |&mut arr| &mut arr.as_mut()
                            .into_t()[start.into_t()..=end.into_t()].into_t());
                    }
                }
            }
            for start in 0..arr.len() {
                assert_eq_api!(arr => |&arr| arr.as_ref()
                    .into_t()[start.into_t()..].into_t());
                assert_eq_api!(arr => |&mut arr| arr.as_mut()
                    .into_t()[start.into_t()..].into_t());
            }
            for end in 0..arr.len() {
                assert_eq_api!(arr => |&arr| arr.as_ref()
                    .into_t()[..end.into_t()].into_t());
                assert_eq_api!(arr => |&mut arr| arr.as_mut()
                    .into_t()[..end.into_t()].into_t());
            }
            if !arr.is_empty() {
                for end in 0..arr.len() - 1 {
                    assert_eq_api!(arr => |&arr| arr.as_ref()
                        .into_t()[..=end.into_t()].into_t());
                    assert_eq_api!(arr => |&mut arr| arr.as_mut()
                        .into_t()[..=end.into_t()].into_t());
                }
            }
            assert_eq_api!(arr => |&arr| {
                #[allow(clippy::into_iter_on_ref)]
                let mut iter = arr.as_ref().into_t().into_iter();
                array_32_from(|| iter.next())
            });
            assert_eq_api!(arr => |&mut arr| {
                #[allow(clippy::into_iter_on_ref)]
                let mut iter: IterMut<'_, _> = arr.as_mut().into_t().into_iter();
                array_32_from(|| iter.next())
            });
        });
        for_in!(for lhs in [[0; 0], [1], [1, 2], [1, 2, 4], [1, 2, 4, 8]] {
            for_in!(for rhs in [[0; 0], [1], [1, 2], [1, 2, 4], [1, 2, 4, 8]] {
                let arrays = (lhs, rhs);
                assert_eq_api!(arrays => |arrays| arrays.0.as_ref().into_t() == arrays.1.as_ref().into_t());
            });
        });
        let default_slice_ref: &[usize] = Default::default();
        let default_ti_slice_ref: &TiSlice<Id, usize> = Default::default();
        assert_eq!(default_slice_ref, &default_ti_slice_ref.raw);
        let default_slice_mut: &mut [usize] = Default::default();
        let default_ti_slice_mut: &mut TiSlice<Id, usize> = Default::default();
        assert_eq!(default_slice_mut, &mut default_ti_slice_mut.raw);
    }

    #[cfg(any(feature = "alloc", feature = "std"))]
    #[test]
    fn alloc_trait_api_compatibility() {
        use alloc::borrow::ToOwned;
        for_in!(for arr in [[0; 0], [1], [1, 2], [1, 2, 4], [1, 2, 4, 8]] {
            assert_eq_api!(arr => |arr| arr.as_ref().into_t().to_owned().as_slice().into_t());
        });
    }

    #[test]
    fn use_non_zero_indecies() {
        use core::num::NonZeroUsize;

        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        struct Id(NonZeroUsize);

        impl From<usize> for Id {
            fn from(value: usize) -> Self {
                Self(NonZeroUsize::new(value + 1).unwrap())
            }
        }

        impl From<Id> for usize {
            fn from(value: Id) -> Self {
                value.0.get() - 1
            }
        }

        assert_eq!(
            core::mem::size_of::<Option<Id>>(),
            core::mem::size_of::<Id>()
        );

        let slice: &TiSlice<Id, usize> = TiSlice::from_ref(&[1, 2, 4, 8, 16]);
        assert_eq!(
            slice.first_key_value(),
            Some((Id(NonZeroUsize::new(1).unwrap()), &1))
        );
    }
}
