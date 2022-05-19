use core::fmt::Debug;
use core::fmt::Display;
use core::fmt::Formatter;
use core::fmt::Result as FmtResult;

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Error {
  InvalidAuthority,
  InvalidFragment,
  InvalidMethodId,
  InvalidMethodName,
  InvalidPath,
  InvalidQuery,
  InvalidScheme,
}

impl Error {
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::InvalidAuthority => "Invalid Authority",
      Self::InvalidFragment => "Invalid Fragment",
      Self::InvalidMethodId => "Invalid Method Id",
      Self::InvalidMethodName => "Invalid Method Name",
      Self::InvalidPath => "Invalid Path",
      Self::InvalidQuery => "Invalid Query",
      Self::InvalidScheme => "Invalid Scheme",
    }
  }
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter) -> FmtResult {
    f.write_str(self.as_str())
  }
}

#[cfg(feature = "std")]
impl ::std::error::Error for Error {}
