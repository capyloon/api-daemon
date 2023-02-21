//! Declare error type for tor-netdir

use thiserror::Error;
use tor_error::HasKind;

/// An error returned by the network directory code
#[derive(Error, Clone, Debug)]
#[non_exhaustive]
pub enum Error {
    /// We don't have enough directory info to build circuits
    #[error("Not enough directory information to build circuits")]
    NotEnoughInfo,
    /// We don't have any directory information.
    #[error("No directory information available")]
    NoInfo,
    /// We have directory information, but it is too expired to use.
    #[error("Directory is expired, and we haven't got a new one yet")]
    DirExpired,
    /// We have directory information, but it is too expired to use.
    #[error("Directory is published too far in the future: Your clock is probably wrong")]
    DirNotYetValid,
}

impl HasKind for Error {
    fn kind(&self) -> tor_error::ErrorKind {
        use tor_error::ErrorKind as EK;
        use Error as E;
        match self {
            E::DirExpired => EK::DirectoryExpired,
            E::DirNotYetValid => EK::ClockSkew,
            E::NotEnoughInfo | E::NoInfo => EK::BootstrapRequired,
        }
    }
}
