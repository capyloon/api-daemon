// Rust language amplification library providing multiple generic trait
// implementations, type wrappers, derive macros and other language enhancements
//
// Written in 2019-2022 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//     Martin Habovstiak <martin.habovstiak@gmail.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

//! Amplifying Rust language capabilities: multiple generic trait
//! implementations, type wrappers, derive macros.
//!
//! Minimum supported rust compiler version (MSRV): 1.46 (stable channel)

#![recursion_limit = "256"]
#![deny(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    unused_mut,
    unused_imports,
    dead_code,
    missing_docs
)]
#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "alloc")]
extern crate alloc;
extern crate core;

#[cfg(feature = "derive")]
extern crate amplify_derive;
#[cfg(feature = "derive")]
pub use amplify_derive::{Wrapper, WrapperMut, Display, AsAny, From, Getters, Error};

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde_crate as serde;

extern crate amplify_num;
#[cfg(any(test, feature = "hex"))]
pub use num::hex;

#[cfg(feature = "stringly_conversions")]
pub extern crate stringly_conversions;
#[cfg(feature = "stringly_conversions")]
pub use stringly_conversions::*;

#[cfg(feature = "proc_attr")]
pub extern crate amplify_syn as proc_attr;
#[cfg(feature = "proc_attr")]
pub use proc_attr::ident;

pub extern crate ascii;

#[macro_use]
mod macros;

#[cfg(feature = "std")]
mod io_util;
pub mod strategy;

mod collection;
pub use collection::*;

mod traits;
pub use traits::*;

pub mod num {
    //! Custom-sized numeric types
    //!
    //! Implementation of a various integer types with custom bit dimension.
    //! These includes:
    //! * large signed and unsigned integers, named *gib int types* (256, 512,
    //!   1024-bit)
    //! * custom sub-8 bit unsigned integers, named *small int types (5-, 6-,
    //!   7-bit)
    //! * 24-bit signed integer.
    //!
    //! The functions here are designed to be fast.

    pub use amplify_num::*;
    #[cfg(feature = "apfloat")]
    pub use amplify_apfloat as apfloat;
}

#[cfg(feature = "std")]
pub use crate::io_util::{IoError, WriteCounter, ConfinedIo};
pub use crate::strategy::Holder;
