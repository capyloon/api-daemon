#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
//! `tor-bytes`: Utilities to decode/encode things into bytes.
//!
//! # Overview
//!
//! The `tor-bytes` crate is part of
//! [Arti](https://gitlab.torproject.org/tpo/core/arti/), a project to
//! implement [Tor](https://www.torproject.org/) in Rust.
//! Other crates in Arti use it to build and handle all the byte-encoded
//! objects from the Tor protocol.  For textual directory items, see
//! the [`tor-netdoc`] crate.
//!
//! This crate is generally useful for encoding and decoding
//! byte-oriented formats that are not regular enough to qualify for
//! serde, and not complex enough to need a full meta-language.  It is
//! probably not suitable for handling anything bigger than a few
//! kilobytes in size.
//!
//! ## Alternatives
//!
//! The Reader/Writer traits in std::io are more appropriate for
//! operations that can fail because of some IO problem.  This crate
//! can't handle that: it is for handling things that are already in
//! memory.
//!
//! TODO: Look into using the "bytes" crate more here.
//!
//! TODO: The "untrusted" crate has similar goals to our [`Reader`],
//! but takes more steps to make sure it can never panic. Perhaps we
//! should see if we can learn any tricks from it.
//!
//! TODO: Do we really want to keep `Reader` as a struct and
//! `Writer` as a trait?
//!
//! # Contents and concepts
//!
//! This crate is structured around four key types:
//!
//! * [`Reader`]: A view of a byte slice, from which data can be decoded.
//! * [`Writer`]: Trait to represent a growable buffer of bytes.
//!   (Vec<u8> and [`bytes::BytesMut`] implement this.)
//! * [`Writeable`]: Trait for an object that can be encoded onto a [`Writer`]
//! * [`Readable`]: Trait for an object that can be decoded from a [`Reader`].
//!
//! Every object you want to encode or decode should implement
//! [`Writeable`] or [`Readable`] respectively.
//!
//! Once you implement these traits, you can use Reader and Writer to
//! handle your type, and other types that are built around it.

// @@ begin lint list maintained by maint/add_warning @@
#![cfg_attr(not(ci_arti_stable), allow(renamed_and_removed_lints))]
#![cfg_attr(not(ci_arti_nightly), allow(unknown_lints))]
#![deny(missing_docs)]
#![warn(noop_method_call)]
#![deny(unreachable_pub)]
#![warn(clippy::all)]
#![deny(clippy::await_holding_lock)]
#![deny(clippy::cargo_common_metadata)]
#![deny(clippy::cast_lossless)]
#![deny(clippy::checked_conversions)]
#![warn(clippy::cognitive_complexity)]
#![deny(clippy::debug_assert_with_mut_call)]
#![deny(clippy::exhaustive_enums)]
#![deny(clippy::exhaustive_structs)]
#![deny(clippy::expl_impl_clone_on_copy)]
#![deny(clippy::fallible_impl_from)]
#![deny(clippy::implicit_clone)]
#![deny(clippy::large_stack_arrays)]
#![warn(clippy::manual_ok_or)]
#![deny(clippy::missing_docs_in_private_items)]
#![deny(clippy::missing_panics_doc)]
#![warn(clippy::needless_borrow)]
#![warn(clippy::needless_pass_by_value)]
#![warn(clippy::option_option)]
#![warn(clippy::rc_buffer)]
#![deny(clippy::ref_option_ref)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![warn(clippy::trait_duplication_in_bounds)]
#![deny(clippy::unnecessary_wraps)]
#![warn(clippy::unseparated_literal_suffix)]
#![deny(clippy::unwrap_used)]
#![allow(clippy::let_unit_value)] // This can reasonably be done for explicitness
#![allow(clippy::significant_drop_in_scrutinee)] // arti/-/merge_requests/588/#note_2812945
//! <!-- @@ end lint list maintained by maint/add_warning @@ -->

mod err;
mod impls;
mod reader;
mod secretbuf;
mod writer;

pub use err::{EncodeError, Error};
pub use reader::Reader;
pub use secretbuf::SecretBuf;
pub use writer::Writer;

use arrayref::array_ref;

/// Result type returned by this crate for [`Reader`]-related methods.
pub type Result<T> = std::result::Result<T, Error>;
/// Result type returned by this crate for [`Writer`]-related methods.
pub type EncodeResult<T> = std::result::Result<T, EncodeError>;

/// Trait for an object that can be encoded onto a Writer by reference.
///
/// Implement this trait in order to make an object writeable.
///
/// Most code won't need to call this directly, but will instead use
/// it implicitly via the Writer::write() method.
///
/// # Example
///
/// ```
/// use tor_bytes::{Writeable, Writer, EncodeResult};
/// #[derive(Debug, Eq, PartialEq)]
/// struct Message {
///   flags: u32,
///   cmd: u8
/// }
///
/// impl Writeable for Message {
///     fn write_onto<B:Writer+?Sized>(&self, b: &mut B) -> EncodeResult<()> {
///         // We'll say that a "Message" is encoded as flags, then command.
///         b.write_u32(self.flags);
///         b.write_u8(self.cmd);
///         Ok(())
///     }
/// }
///
/// let msg = Message { flags: 0x43, cmd: 0x07 };
/// let mut writer: Vec<u8> = Vec::new();
/// writer.write(&msg);
/// assert_eq!(writer, &[0x00, 0x00, 0x00, 0x43, 0x07 ]);
/// ```
pub trait Writeable {
    /// Encode this object into the writer `b`.
    fn write_onto<B: Writer + ?Sized>(&self, b: &mut B) -> EncodeResult<()>;
}

/// Trait for an object that can be encoded and consumed by a Writer.
///
/// Implement this trait in order to make an object that can be
/// written more efficiently by absorbing it into the writer.
///
/// Most code won't need to call this directly, but will instead use
/// it implicitly via the Writer::write_and_consume() method.
pub trait WriteableOnce: Sized {
    /// Encode this object into the writer `b`, and consume it.
    fn write_into<B: Writer + ?Sized>(self, b: &mut B) -> EncodeResult<()>;
}

impl<W: Writeable + Sized> WriteableOnce for W {
    fn write_into<B: Writer + ?Sized>(self, b: &mut B) -> EncodeResult<()> {
        self.write_onto(b)
    }
}

// ----------------------------------------------------------------------

/// Trait for an object that can be extracted from a Reader.
///
/// Implement this trait in order to make an object that can (maybe)
/// be decoded from a reader.
//
/// Most code won't need to call this directly, but will instead use
/// it implicitly via the Reader::extract() method.
///
/// # Example
///
/// ```
/// use tor_bytes::{Readable,Reader,Result};
/// #[derive(Debug, Eq, PartialEq)]
/// struct Message {
///   flags: u32,
///   cmd: u8
/// }
///
/// impl Readable for Message {
///     fn take_from(r: &mut Reader<'_>) -> Result<Self> {
///         // A "Message" is encoded as flags, then command.
///         let flags = r.take_u32()?;
///         let cmd = r.take_u8()?;
///         Ok(Message{ flags, cmd })
///     }
/// }
///
/// let encoded = [0x00, 0x00, 0x00, 0x43, 0x07 ];
/// let mut reader = Reader::from_slice(&encoded);
/// let m: Message = reader.extract()?;
/// assert_eq!(m, Message { flags: 0x43, cmd: 0x07 });
/// reader.should_be_exhausted()?; // make sure there are no bytes left over
/// # Result::Ok(())
/// ```
pub trait Readable: Sized {
    /// Try to extract an object of this type from a Reader.
    ///
    /// Implementations should generally try to be efficient: this is
    /// not the right place to check signatures or perform expensive
    /// operations.  If you have an object that must not be used until
    /// it is finally validated, consider making this function return
    /// a wrapped type that can be unwrapped later on once it gets
    /// checked.
    fn take_from(b: &mut Reader<'_>) -> Result<Self>;
}

// ----------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn writer() {
        let mut v: Vec<u8> = Vec::new();
        v.write_u8(0x57);
        v.write_u16(0x6520);
        v.write_u32(0x68617665);
        v.write_u64(0x2061206d61636869);
        v.write_all(b"ne in a plexiglass dome");
        v.write_zeros(3);
        assert_eq!(&v[..], &b"We have a machine in a plexiglass dome\0\0\0"[..]);
    }
}
