#[cfg(feature = "alloc")]
use alloc::string::String;

use core::ops::Range;
use core::ops::RangeFrom;
use core::ops::RangeTo;

use crate::did::DID;
use crate::error::Error;
use crate::error::Result;
use crate::input::Input;

#[derive(Clone, Debug)]
pub struct Core {
  pub(crate) method: u32,           // Includes leading :
  pub(crate) method_id: u32,        // Includes leading :
  pub(crate) path: u32,             // Includes leading /
  pub(crate) query: Option<u32>,    // Includes leading ?
  pub(crate) fragment: Option<u32>, // Includes leading #
}

impl Core {
  const fn new() -> Self {
    Self {
      method: 0,
      method_id: 0,
      path: 0,
      query: None,
      fragment: None,
    }
  }

  pub(crate) fn authority<'a>(&self, data: &'a str) -> &'a str {
    self.slice(data, self.method + 1..self.path)
  }

  pub(crate) fn method<'a>(&self, data: &'a str) -> &'a str {
    self.slice(data, self.method + 1..self.method_id)
  }

  pub(crate) fn method_id<'a>(&self, data: &'a str) -> &'a str {
    self.slice(data, self.method_id + 1..self.path)
  }

  pub(crate) fn path<'a>(&self, data: &'a str) -> &'a str {
    match (self.query, self.fragment) {
      (None, None) => self.slice(data, self.path..),
      (Some(index), _) | (None, Some(index)) => self.slice(data, self.path..index),
    }
  }

  pub(crate) fn query<'a>(&self, data: &'a str) -> Option<&'a str> {
    match (self.query, self.fragment) {
      (None, _) => None,
      (Some(query), None) => Some(self.slice(data, query + 1..)),
      (Some(query), Some(fragment)) => Some(self.slice(data, query + 1..fragment)),
    }
  }

  pub(crate) fn fragment<'a>(&self, data: &'a str) -> Option<&'a str> {
    self
      .fragment
      .map(|fragment| self.slice(data, fragment + 1..))
  }

  pub(crate) fn query_pairs<'a>(&self, data: &'a str) -> form_urlencoded::Parse<'a> {
    form_urlencoded::parse(self.query(data).unwrap_or_default().as_bytes())
  }

  pub(crate) fn set_method(&mut self, buffer: &mut String, value: &str) {
    let int: Int = Int::new(self.method_id, self.method + 1 + value.len() as u32);

    buffer.replace_range(self.method as usize + 1..self.method_id as usize, value);

    self.method_id = int.add(self.method_id);
    self.path = int.add(self.path);
    self.query = int.try_add(self.query);
    self.fragment = int.try_add(self.fragment);
  }

  pub(crate) fn set_method_id(&mut self, buffer: &mut String, value: &str) {
    let int: Int = Int::new(self.path, self.method_id + 1 + value.len() as u32);

    buffer.replace_range(self.method_id as usize + 1..self.path as usize, value);

    self.path = int.add(self.path);
    self.query = int.try_add(self.query);
    self.fragment = int.try_add(self.fragment);
  }

  pub(crate) fn set_path(&mut self, buffer: &mut String, value: &str) {
    let end: u32 = self
      .query
      .or(self.fragment)
      .unwrap_or_else(|| buffer.len() as u32);

    let int: Int = Int::new(end, self.path + value.len() as u32);

    buffer.replace_range(self.path as usize..end as usize, value);

    self.query = int.try_add(self.query);
    self.fragment = int.try_add(self.fragment);
  }

  pub(crate) fn set_query(&mut self, buffer: &mut String, value: Option<&str>) {
    match (self.query, self.fragment, value) {
      (Some(query), None, Some(value)) => {
        buffer.replace_range(query as usize + 1.., value);
      }
      (None, Some(fragment), Some(value)) => {
        self.query = Some(fragment);
        self.fragment = Some(fragment + value.len() as u32 + 1);

        buffer.insert_str(fragment as usize, "?");
        buffer.insert_str(fragment as usize + 1, value);
      }
      (Some(query), Some(fragment), Some(value)) => {
        self.fragment = Some(query + value.len() as u32 + 1);

        buffer.replace_range(query as usize + 1..fragment as usize, value);
      }
      (None, None, Some(value)) => {
        self.query = Some(buffer.len() as u32);
        buffer.push('?');
        buffer.push_str(value);
      }
      (Some(query), None, None) => {
        self.query = None;
        buffer.truncate(query as usize);
      }
      (Some(query), Some(fragment), None) => {
        self.query = None;
        self.fragment = Some(fragment - (fragment - query));

        buffer.replace_range(query as usize..fragment as usize, "");
      }
      (None, _, None) => {
        // do nothing
      }
    }
  }

  pub(crate) fn set_fragment(&mut self, buffer: &mut String, value: Option<&str>) {
    if let Some(index) = self.fragment {
      buffer.truncate(index as usize);
    }

    if let Some(value) = value {
      self.fragment = Some(buffer.len() as u32);
      buffer.push('#');
      buffer.push_str(value);
    } else {
      self.fragment = None;
    }
  }

  fn slice<'a>(&self, data: &'a str, range: impl SliceExt) -> &'a str {
    range.slice(data)
  }

  /// Parse a DID URL adhering to the following format:
  ///
  ///   did                = "did:" method-name ":" method-specific-id
  ///   method-name        = 1*method-char
  ///   method-char        = %x61-7A / DIGIT
  ///   method-specific-id = *( *idchar ":" ) 1*idchar
  ///   idchar             = ALPHA / DIGIT / "." / "-" / "_"
  ///
  ///   did-url            = did path-abempty [ "?" query ] [ "#" fragment ]
  ///
  ///   path-abempty       = *( "/" segment )
  ///   segment            = *pchar
  ///   pchar              = unreserved / pct-encoded / sub-delims / ":" / "@"
  ///   unreserved         = ALPHA / DIGIT / "-" / "." / "_" / "~"
  ///   pct-encoded        = "%" HEXDIG HEXDIG
  ///   sub-delims         = "!" / "$" / "&" / "'" / "(" / ")" / "*" / "+" / "," / ";" / "="
  ///
  ///   query              = *( pchar / "/" / "?" )
  ///
  ///   fragment           = *( pchar / "/" / "?" )
  ///
  pub(crate) fn parse(data: impl AsRef<str>) -> Result<Self> {
    let mut this: Self = Self::new();
    let mut input: Input = Input::new(data.as_ref());

    this.parse_scheme(&mut input)?;
    this.parse_method(&mut input)?;
    this.parse_method_id(&mut input)?;
    this.parse_path(&mut input)?;
    this.parse_query(&mut input)?;
    this.parse_fragment(&mut input)?;

    if this.method(data.as_ref()).is_empty() {
      return Err(Error::InvalidMethodName);
    }

    if this.method_id(data.as_ref()).is_empty() {
      return Err(Error::InvalidMethodId);
    }

    Ok(this)
  }

  pub(crate) fn parse_relative(data: impl AsRef<str>) -> Result<Self> {
    let mut this: Self = Self::new();
    let mut input: Input = Input::new(data.as_ref());

    this.parse_path(&mut input)?;
    this.parse_query(&mut input)?;
    this.parse_fragment(&mut input)?;

    Ok(this)
  }

  fn parse_scheme(&mut self, input: &mut Input) -> Result<()> {
    if input.exhausted() {
      return Err(Error::InvalidScheme);
    }

    if !matches!(input.take(3), Some(DID::SCHEME)) {
      return Err(Error::InvalidScheme);
    }

    Ok(())
  }

  fn parse_method(&mut self, input: &mut Input) -> Result<()> {
    if matches!(input.peek(), Some(':')) {
      input.next();
    } else {
      return Err(Error::InvalidMethodName);
    }

    self.method = input.index() - 1;

    loop {
      match input.peek() {
        Some(':') | None => break,
        Some(ch) if char_method(ch) => {}
        _ => return Err(Error::InvalidMethodName),
      }

      input.next();
    }

    Ok(())
  }

  fn parse_method_id(&mut self, input: &mut Input) -> Result<()> {
    if matches!(input.peek(), Some(':')) {
      input.next();
    } else {
      return Err(Error::InvalidMethodId);
    }

    self.method_id = input.index() - 1;

    loop {
      match input.peek() {
        Some('/') | Some('?') | Some('#') | None => break,
        Some(ch) if char_method_id(ch) => {}
        _ => return Err(Error::InvalidMethodId),
      }

      input.next();
    }

    Ok(())
  }

  fn parse_path(&mut self, input: &mut Input) -> Result<()> {
    self.path = input.index();

    if matches!(input.peek(), Some('?') | Some('#') | None) {
      return Ok(());
    }

    loop {
      match input.peek() {
        Some('?') | Some('#') | None => break,
        Some(ch) if char_path(ch) => {}
        _ => return Err(Error::InvalidPath),
      }

      input.next();
    }

    Ok(())
  }

  fn parse_query(&mut self, input: &mut Input) -> Result<()> {
    if matches!(input.peek(), Some('#') | None) {
      return Ok(());
    }

    if matches!(input.peek(), Some('?')) {
      input.next();
    } else {
      return Err(Error::InvalidQuery);
    }

    self.query = Some(input.index() - 1);

    loop {
      match input.peek() {
        Some('#') | None => break,
        Some(ch) if char_query(ch) => {}
        _ => return Err(Error::InvalidQuery),
      }

      input.next();
    }

    Ok(())
  }

  fn parse_fragment(&mut self, input: &mut Input) -> Result<()> {
    if input.exhausted() {
      return Ok(());
    }

    if matches!(input.peek(), Some('#')) {
      input.next();
    } else {
      return Err(Error::InvalidFragment);
    }

    self.fragment = Some(input.index() - 1);

    loop {
      match input.peek() {
        None => break,
        Some(ch) if char_fragment(ch) => {}
        _ => return Err(Error::InvalidFragment),
      }

      input.next();
    }

    Ok(())
  }
}

// =============================================================================
//
// =============================================================================

#[inline(always)]
const fn char_method(ch: char) -> bool {
  matches!(ch, '0'..='9' | 'a'..='z')
}

#[inline(always)]
const fn char_method_id(ch: char) -> bool {
  matches!(ch, '0'..='9' | 'a'..='z' | 'A'..='Z' | '.' | '-' | '_' | ':')
}

#[inline(always)]
#[rustfmt::skip]
const fn char_path(ch: char) -> bool {
  char_method_id(ch) || matches!(ch, '~' | '!' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | ';' | '=' | '@' | '/' /* | '%' */)
}

#[inline(always)]
const fn char_query(ch: char) -> bool {
  char_path(ch) || ch == '?'
}

#[inline(always)]
const fn char_fragment(ch: char) -> bool {
  char_path(ch) || ch == '?'
}

// =============================================================================
//
// =============================================================================

pub trait SliceExt {
  fn slice<'a>(&self, string: &'a str) -> &'a str;
}

impl SliceExt for Range<u32> {
  fn slice<'a>(&self, string: &'a str) -> &'a str {
    &string[self.start as usize..self.end as usize]
  }
}

impl SliceExt for RangeFrom<u32> {
  fn slice<'a>(&self, string: &'a str) -> &'a str {
    &string[self.start as usize..]
  }
}

impl SliceExt for RangeTo<u32> {
  fn slice<'a>(&self, string: &'a str) -> &'a str {
    &string[..self.end as usize]
  }
}

// =============================================================================
//
// =============================================================================

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum Int {
  N(u32),
  P(u32),
}

impl Int {
  const fn new(old: u32, new: u32) -> Self {
    if old > new {
      Self::N(old - new)
    } else {
      Self::P(new - old)
    }
  }

  const fn add(self, other: u32) -> u32 {
    match self {
      Self::N(int) => other - int,
      Self::P(int) => other + int,
    }
  }

  const fn try_add(self, other: Option<u32>) -> Option<u32> {
    match other {
      Some(other) => Some(self.add(other)),
      None => None,
    }
  }
}
