//! Cryptographic functions of the tor protocol.
//!
//! There are three sub-modules here:
//!
//!   * `cell` implements relay crypto as used on circuits.
//!   * `handshake` implements the ntor handshake.
//!   * `ll` provides building blocks for other parts of the protocol.

pub(crate) mod cell;
pub(crate) mod handshake;
pub(crate) mod ll;
#[cfg(test)]
mod testing;
