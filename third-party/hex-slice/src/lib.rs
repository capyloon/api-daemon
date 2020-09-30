//! The purpose of this crate is to extend the `UpperHex` and `LowerHex`
//! traits to slices, as well as the integers it is currently implemented for.
//!
//! # Examples
//!
//! ```rust
//! extern crate hex_slice;
//! use hex_slice::AsHex;
//!
//! fn main() {
//!     let foo = vec![0u32, 1 ,2 ,3];
//!     println!("{:x}", foo.as_hex());
//! }
//! ```

#![no_std]

use core::fmt;
use core::fmt::Write;

pub struct Hex<'a, T: 'a>(&'a [T]);

pub struct PlainHex<'a, T: 'a> {
    slice: &'a [T],
    with_spaces: bool,
}

pub trait AsHex {
    type Item;
    fn as_hex<'a>(&'a self) -> Hex<'a, Self::Item>;

    fn plain_hex<'a>(&'a self, with_spaces: bool) -> PlainHex<'a, Self::Item>;
}

fn fmt_inner_hex<T, F: Fn(&T, &mut fmt::Formatter) -> fmt::Result>(slice: &[T], f: &mut fmt::Formatter, fmt_fn: F, with_spaces: bool) -> fmt::Result {
    for (i, val) in slice.iter().enumerate() {
        if with_spaces && i > 0 {
            f.write_char(' ')?;
        }
        fmt_fn(val, f)?;
    }
    Ok(())
}

impl<'a, T> Hex<'a, T> {
    pub fn hex(slice: &'a [T]) -> Hex<'a, T> {
        Hex(slice)
    }
}

impl<'a, T: fmt::LowerHex> fmt::LowerHex for Hex<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        fmt_inner_hex(self.0, f, fmt::LowerHex::fmt, true)?;
        write!(f, "]")
    }
}

impl<'a, T: fmt::UpperHex> fmt::UpperHex for Hex<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        fmt_inner_hex(self.0, f, fmt::UpperHex::fmt, true)?;
        write!(f, "]")
    }
}

impl<'a, T: fmt::LowerHex> fmt::LowerHex for PlainHex<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt_inner_hex(self.slice, f, fmt::LowerHex::fmt, self.with_spaces)
    }
}

impl<'a, T: fmt::UpperHex> fmt::UpperHex for PlainHex<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt_inner_hex(self.slice, f, fmt::UpperHex::fmt, self.with_spaces)
    }
}

impl<T> AsHex for [T] {
    type Item = T;
    fn as_hex<'a>(&'a self) -> Hex<'a, Self::Item> {
        Hex::hex(self)
    }

    fn plain_hex<'a>(&'a self, with_spaces: bool) -> PlainHex<'a, Self::Item> {
        PlainHex {
            slice: self,
            with_spaces,
        }
    }
}
