//! # did
//!
//! An implementation of [DID Identifiers](https://www.w3.org/TR/did-core/#identifier) for the [Rust](https://www.rust-lang.org/) programming language.
//!
//! ### References
//!
//! - [DID Core](https://www.w3.org/TR/did-core/)
//!
#![no_std]

#[cfg(not(feature = "alloc"))]
compile_error!("This crate does not yet support environments without liballoc.");

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

mod core;
mod did;
mod error;
mod input;

pub use self::did::DID;
pub use self::error::Error;
pub use self::error::Result;

/// A helper macro to assist with the construction of [`DID`]s.
#[macro_export]
macro_rules! did {
  ($did:expr) => {
    $crate::DID::parse($did).unwrap()
  };
}
