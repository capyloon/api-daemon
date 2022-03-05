use ::std::collections::vec_deque;
#[cfg(feature = "fused")]
use ::std::iter::FusedIterator;

use ::BoundedVecDeque;

macro_rules! forward_iterator_impls {
    ( impl($($param:tt)*), $impl_type:ty, $item_type:ty ) => {
        impl<$($param)*> Iterator for $impl_type {
            type Item = $item_type;

            fn next(&mut self) -> Option<Self::Item> {
                self.iter.next()
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                self.iter.size_hint()
            }

            fn count(self) -> usize {
                self.iter.count()
            }

            fn last(self) -> Option<Self::Item> {
                self.iter.last()
            }

            fn nth(&mut self, n: usize) -> Option<Self::Item> {
                self.iter.nth(n)
            }

            fn fold<A, F>(self, init: A, function: F) -> A
            where F: FnMut(A, Self::Item) -> A {
                self.iter.fold(init, function)
            }
        }

        impl<$($param)*> DoubleEndedIterator for $impl_type {
            fn next_back(&mut self) -> Option<Self::Item> {
                self.iter.next_back()
            }
        }

        impl<$($param)*> ExactSizeIterator for $impl_type {
            fn len(&self) -> usize {
                self.iter.len()
            }
        }

        /// This iterator's `next()` will continue to return `None` when exhausted.
        ///
        /// # Availability
        ///
        /// This trait impl requires [the `fused` feature]. However, the guarantee exists even when
        /// the trait impl is not present.
        ///
        /// [the `fused` feature]: index.html#features
        #[cfg(feature = "fused")]
        impl<$($param)*> FusedIterator for $impl_type {}
    };
}

/// An iterator over references to elements in a [`BoundedVecDeque`].
///
/// This type is returned by [`BoundedVecDeque::iter()`]. See its documentation.
///
/// [`BoundedVecDeque`]: struct.BoundedVecDeque.html
/// [`BoundedVecDeque::iter()`]: struct.BoundedVecDeque.html#method.iter
#[derive(Debug)]
pub struct Iter<'a, T: 'a> {
    pub(crate) iter: vec_deque::Iter<'a, T>,
}

forward_iterator_impls!(impl('a, T), Iter<'a, T>, &'a T);

// `vec_deque::Iter` is `Clone` even when `T` isn't, so implement `Clone` manually instead of
// deriving.
impl<'a, T> Clone for Iter<'a, T> {
    fn clone(&self) -> Self {
        Iter {
            iter: self.iter.clone(),
        }
    }
}

/// An iterator over mutable references to elements in a [`BoundedVecDeque`].
///
/// This type is returned by [`BoundedVecDeque::iter_mut()`]. See its documentation.
///
/// [`BoundedVecDeque`]: struct.BoundedVecDeque.html
/// [`BoundedVecDeque::iter_mut()`]: struct.BoundedVecDeque.html#method.iter_mut
#[derive(Debug)]
pub struct IterMut<'a, T: 'a> {
    pub(crate) iter: vec_deque::IterMut<'a, T>,
}

forward_iterator_impls!(impl('a, T: 'a), IterMut<'a, T>, &'a mut T);

/// An owning iterator over elements from a [`BoundedVecDeque`].
///
/// This type is returned by [`BoundedVecDeque::into_iter()`]. See its documentation.
///
/// [`BoundedVecDeque`]: struct.BoundedVecDeque.html
/// [`BoundedVecDeque::into_iter()`]: struct.BoundedVecDeque.html#method.into_iter
#[derive(Debug, Clone)]
pub struct IntoIter<T> {
    pub(crate) iter: vec_deque::IntoIter<T>,
}

forward_iterator_impls!(impl(T), IntoIter<T>, T);

/// A draining iterator over elements from a [`BoundedVecDeque`].
///
/// This type is returned by [`BoundedVecDeque::drain()`]. See its documentation.
///
/// [`BoundedVecDeque`]: struct.BoundedVecDeque.html
/// [`BoundedVecDeque::drain()`]: struct.BoundedVecDeque.html#method.drain
#[derive(Debug)]
pub struct Drain<'a, T: 'a> {
    pub(crate) iter: vec_deque::Drain<'a, T>,
}

forward_iterator_impls!(impl('a, T: 'a), Drain<'a, T>, T);

/// A draining iterator over elements from a [`BoundedVecDeque`].
///
/// This type is returned by [`BoundedVecDeque::append()`]. See its documentation.
///
/// [`BoundedVecDeque`]: struct.BoundedVecDeque.html
/// [`BoundedVecDeque::append()`]: struct.BoundedVecDeque.html#method.append
#[derive(Debug)]
pub struct Append<'a, T: 'a> {
    pub(crate) source: &'a mut BoundedVecDeque<T>,
    pub(crate) destination: &'a mut BoundedVecDeque<T>,
}

impl<'a, T: 'a> Drop for Append<'a, T> {
    fn drop(&mut self) {
        // Run self until the end to make sure all the values make it from source to destination.
        while let Some(_) = self.next() {
            continue
        }
    }
}

impl<'a, T: 'a> Iterator for Append<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.source.pop_front() {
                Some(value) => match self.destination.push_back(value) {
                    Some(displaced_value) => return Some(displaced_value),
                    None => continue,
                },
                None => return None,
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<'a, T: 'a> ExactSizeIterator for Append<'a, T> {
    fn len(&self) -> usize {
        self.destination
            .max_len()
            .saturating_sub(self.destination.len())
            .saturating_sub(self.source.len())
    }
}

/// This iterator's `next()` will continue to return `None` when exhausted.
///
/// # Availability
///
/// This trait impl requires [the `fused` feature]. However, the guarantee exists even when the
/// trait impl is not present.
///
/// [the `fused` feature]: index.html#features
#[cfg(feature = "fused")]
impl<'a, T: 'a> FusedIterator for Append<'a, T> {}
