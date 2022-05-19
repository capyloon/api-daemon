#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::string::ToString as _;
use core::cmp::Ordering;
use core::convert::TryFrom;
use core::fmt::Debug;
use core::fmt::Display;
use core::fmt::Formatter;
use core::fmt::Result as FmtResult;
use core::hash::Hash;
use core::hash::Hasher;
use core::str::FromStr;

use crate::core::Core;
use crate::error::Error;
use crate::error::Result;

#[derive(Clone, Copy)]
pub struct Inspect<'a>(&'a DID);

impl Debug for Inspect<'_> {
  fn fmt(&self, f: &mut Formatter) -> FmtResult {
    f.debug_struct("DID")
      .field("method", &self.0.method())
      .field("method_id", &self.0.method_id())
      .field("path", &self.0.path())
      .field("query", &self.0.query())
      .field("fragment", &self.0.fragment())
      .finish()
  }
}

/// A Decentralized Identifier (DID).
///
/// [More Info (W3C DID Core)](https://www.w3.org/TR/did-core/)
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(into = "String", try_from = "String"))]
pub struct DID {
  data: String,
  core: Core,
}

impl DID {
  /// The URL scheme for Decentralized Identifiers.
  pub const SCHEME: &'static str = "did";

  /// Parses a [`DID`] from the provided `input`.
  ///
  /// # Errors
  ///
  /// Returns `Err` if any DID segments are invalid.
  pub fn parse(input: impl AsRef<str>) -> Result<Self> {
    Ok(Self {
      data: input.as_ref().to_string(),
      core: Core::parse(input)?,
    })
  }

  /// Returns a wrapped `DID` with a more detailed `Debug` implementation.
  #[inline]
  pub const fn inspect(&self) -> Inspect {
    Inspect(self)
  }

  /// Returns the serialized [`DID`].
  ///
  /// This is fast since the serialized value is stored in the [`DID`].
  #[inline]
  pub fn as_str(&self) -> &str {
    &*self.data
  }

  /// Consumes the [`DID`] and returns the serialization.
  #[cfg(feature = "alloc")]
  #[inline]
  pub fn into_string(self) -> String {
    self.data
  }

  /// Returns the [`DID`] scheme. See [`DID::SCHEME`].
  #[inline]
  pub const fn scheme(&self) -> &'static str {
    DID::SCHEME
  }

  /// Returns the [`DID`] authority.
  #[inline]
  pub fn authority(&self) -> &str {
    self.core.authority(self.as_str())
  }

  /// Returns the [`DID`] method name.
  #[inline]
  pub fn method(&self) -> &str {
    self.core.method(self.as_str())
  }

  /// Returns the [`DID`] method-specific ID.
  #[inline]
  pub fn method_id(&self) -> &str {
    self.core.method_id(self.as_str())
  }

  /// Returns the [`DID`] path.
  #[inline]
  pub fn path(&self) -> &str {
    self.core.path(self.as_str())
  }

  /// Returns the [`DID`] method query, if any.
  #[inline]
  pub fn query(&self) -> Option<&str> {
    self.core.query(self.as_str())
  }

  /// Returns the [`DID`] method fragment, if any.
  #[inline]
  pub fn fragment(&self) -> Option<&str> {
    self.core.fragment(self.as_str())
  }

  /// Parses the [`DID`] query and returns an iterator of (key, value) pairs.
  #[inline]
  pub fn query_pairs(&self) -> form_urlencoded::Parse {
    self.core.query_pairs(self.as_str())
  }

  /// Change the method of the [`DID`].
  #[inline]
  pub fn set_method(&mut self, value: impl AsRef<str>) {
    self.core.set_method(&mut self.data, value.as_ref());
  }

  /// Change the method-specific-id of the [`DID`].
  #[inline]
  pub fn set_method_id(&mut self, value: impl AsRef<str>) {
    self.core.set_method_id(&mut self.data, value.as_ref());
  }

  /// Change the path of the [`DID`].
  #[inline]
  pub fn set_path(&mut self, value: impl AsRef<str>) {
    self.core.set_path(&mut self.data, value.as_ref());
  }

  /// Change the query of the [`DID`].
  ///
  /// No serialization is performed.
  #[inline]
  pub fn set_query(&mut self, value: Option<&str>) {
    self.core.set_query(&mut self.data, value);
  }

  /// Change the fragment of the [`DID`].
  ///
  /// No serialization is performed.
  #[inline]
  pub fn set_fragment(&mut self, value: Option<&str>) {
    self.core.set_fragment(&mut self.data, value);
  }

  /// Creates a new [`DID`] by joining `self` with the relative DID `other`.
  ///
  /// # Errors
  ///
  /// Returns `Err` if any base or relative DID segments are invalid.
  #[cfg(feature = "alloc")]
  pub fn join(&self, other: impl AsRef<str>) -> Result<Self> {
    let data: &str = other.as_ref();
    let core: Core = Core::parse_relative(data)?;

    resolution::transform_references(self, (data, &core))
  }
}

impl Hash for DID {
  fn hash<H>(&self, hasher: &mut H)
  where
    H: Hasher,
  {
    self.as_str().hash(hasher)
  }
}

impl PartialEq for DID {
  fn eq(&self, other: &Self) -> bool {
    self.as_str() == other.as_str()
  }
}

impl Eq for DID {}

impl PartialOrd for DID {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    self.as_str().partial_cmp(other.as_str())
  }
}

impl Ord for DID {
  fn cmp(&self, other: &Self) -> Ordering {
    self.as_str().cmp(other.as_str())
  }
}

impl PartialEq<str> for DID {
  fn eq(&self, other: &str) -> bool {
    self.as_str() == other
  }
}

impl PartialEq<&'_ str> for DID {
  fn eq(&self, other: &&'_ str) -> bool {
    self == *other
  }
}

impl Debug for DID {
  fn fmt(&self, f: &mut Formatter) -> FmtResult {
    f.write_fmt(format_args!("{:?}", self.as_str()))
  }
}

impl Display for DID {
  fn fmt(&self, f: &mut Formatter) -> FmtResult {
    f.write_fmt(format_args!("{}", self.as_str()))
  }
}

impl AsRef<str> for DID {
  fn as_ref(&self) -> &str {
    self.data.as_ref()
  }
}

impl FromStr for DID {
  type Err = Error;

  fn from_str(string: &str) -> Result<Self, Self::Err> {
    Self::parse(string)
  }
}

#[cfg(feature = "alloc")]
impl TryFrom<String> for DID {
  type Error = Error;

  fn try_from(other: String) -> Result<Self, Self::Error> {
    Self::parse(other)
  }
}

#[cfg(feature = "alloc")]
impl From<DID> for String {
  fn from(other: DID) -> Self {
    other.into_string()
  }
}

// =============================================================================
// Reference Resolution
// See RFC 3986 - https://tools.ietf.org/html/rfc3986#section-5
// =============================================================================

#[cfg(feature = "alloc")]
mod resolution {
  use alloc::borrow::Cow;
  use core::fmt::Display;
  use core::fmt::Formatter;
  use core::fmt::Result as FmtResult;
  use core::str::from_utf8_unchecked;

  use crate::core::Core;
  use crate::did::DID;
  use crate::error::Error;
  use crate::error::Result;

  #[derive(Debug)]
  #[repr(transparent)]
  pub struct Path<'a>(Cow<'a, str>);

  impl<'a> Path<'a> {
    pub const fn new() -> Self {
      Self(Cow::Borrowed(""))
    }

    pub fn push(&mut self, value: impl AsRef<[u8]>) {
      self
        .0
        .to_mut()
        .push_str(unsafe { from_utf8_unchecked(value.as_ref()) });
    }

    pub fn pop(&mut self) {
      if self.0.is_empty() {
        return;
      }

      if let Some(index) = self.0.rfind('/') {
        self.0.to_mut().replace_range(index.., "");
      }
    }
  }

  impl<'a> From<Path<'a>> for Cow<'a, str> {
    fn from(other: Path<'a>) -> Self {
      other.0
    }
  }

  impl Display for Path<'_> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
      Display::fmt(&self.0, f)
    }
  }

  /// Transform References.
  ///
  /// Transforms a DID reference into its target DID.
  ///
  /// [More Info](https://tools.ietf.org/html/rfc3986#section-5.2.2)
  #[allow(non_snake_case)]
  pub fn transform_references(base: &DID, (data, core): (&str, &Core)) -> Result<DID> {
    let P: &str = core.path(data);
    let Q: Option<&str> = core.query(data);

    let mut T: DID = base.clone();

    if P.is_empty() {
      T.set_path(base.path());
      T.set_query(Q.or_else(|| base.query()));
    } else {
      if P.starts_with('/') {
        T.set_path(remove_dot_segments(P));
      } else {
        T.set_path(remove_dot_segments(&merge_paths(base, P)?));
      }

      T.set_query(Q);
    }

    T.set_method(base.method()); // TODO: Remove? This in inherited via clone
    T.set_method_id(base.method_id()); // TODO: Remove? This in inherited via clone
    T.set_fragment(core.fragment(data));

    Ok(T)
  }

  /// Merge Paths.
  ///
  /// Merges a relative-path reference with the path of the base DID.
  ///
  /// [More Info](https://tools.ietf.org/html/rfc3986#section-5.2.3)
  pub fn merge_paths<'a>(base: &'a DID, data: &'a str) -> Result<Cow<'a, str>> {
    // Ensure the base DID has an authority component.
    //
    // The DID authority is `<method>:<method-specific-id>` so it should always
    // be present for non-relative DIDs.
    if base.method().is_empty() || base.method_id().is_empty() {
      return Err(Error::InvalidAuthority);
    }

    // 1. If the base URI has a defined authority component and an empty
    // path, then return a string consisting of "/" concatenated with the
    // reference's path.

    if base.path().is_empty() {
      return Ok(data.into());
    }

    // 2. Return a string consisting of the reference's path component
    // appended to all but the last segment of the base URI's path (i.e.,
    // excluding any characters after the right-most "/" in the base URI
    // path, or excluding the entire base URI path if it does not contain
    // any "/" characters).

    let mut path: &str = base.path();

    if let Some(index) = path.rfind('/') {
      path = &path[..=index];
    }

    Ok([path, data].join("").into())
  }

  /// Remove Dot Segments.
  ///
  /// [More Info](https://tools.ietf.org/html/rfc3986#section-5.2.4)
  pub fn remove_dot_segments(path: &str) -> Cow<str> {
    fn next_segment(input: impl AsRef<[u8]>) -> Option<usize> {
      match input.as_ref() {
        [b'/', input @ ..] => next_segment(input).map(|index| index + 1),
        input => input.iter().position(|byte| *byte == b'/'),
      }
    }

    let mut output: Path = Path::new();
    let mut input: &[u8] = path.as_bytes();

    loop {
      match input {
        // Remove prefix ../
        [b'.', b'.', b'/', ..] => {
          input = &input[3..];
        }
        // Remove prefix ./
        [b'.', b'/', ..] => {
          input = &input[2..];
        }
        // Replace prefix /./
        [b'/', b'.', b'/', ..] => {
          input = &input[2..];
        }
        // Replace prefix /.
        [b'/', b'.'] => {
          input = &input[..1];
        }
        // Replace prefix /../
        [b'/', b'.', b'.', b'/', ..] => {
          input = &input[3..];
          output.pop();
        }
        // Replace prefix /..
        [b'/', b'.', b'.'] => {
          input = &input[..2];
          output.pop();
        }
        // Remove .
        [b'.'] => {
          input = &input[1..];
        }
        // Remove ..
        [b'.', b'.'] => {
          input = &input[2..];
        }
        _ => {
          if let Some(index) = next_segment(input) {
            output.push(&input[..index]);
            input = &input[index..];
          } else {
            output.push(input);
            break;
          }
        }
      }
    }

    output.into()
  }
}
