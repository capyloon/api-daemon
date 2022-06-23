//! Guessing of MIME types by file extension.
//!
//! Uses a static list of file-extension : MIME type mappings.
//!
//! ```
//! // the file doesn't have to exist, it just looks at the path
//! let guess = new_mime_guess::from_path("some_file.gif");
//! assert_eq!(guess.first(), Some(mime::IMAGE_GIF));
//!
//! ```
//!
//! #### Note: MIME Types Returned Are Not Stable/Guaranteed
//! The media types returned for a given extension are not considered to be part of the crate's
//! stable API and are often updated in patch <br /> (`x.y.[z + 1]`) releases to be as correct as
//! possible.
//!
//! Additionally, only the extensions of paths/filenames are inspected in order to guess the MIME
//! type. The file that may or may not reside at that path may or may not be a valid file of the
//! returned MIME type.  Be wary of unsafe or un-validated assumptions about file structure or
//! length.
#[forbid(unsafe_code)]
mod mime_types;

use std::ffi::OsStr;
use std::iter::FusedIterator;
use std::path::Path;
use std::{iter, slice};

pub use mime::Mime;

#[cfg(feature = "phf-map")]
#[path = "impl_phf.rs"]
mod impl_;

#[cfg(not(feature = "phf-map"))]
#[path = "impl_bin_search.rs"]
mod impl_;
#[cfg(test)]
mod tests;

/// A "guess" of the MIME/Media Type(s) of an extension or path as one or more
/// [`Mime`](struct.Mime.html) instances.
///
/// ### Note: Ordering
/// A given file format may have one or more applicable Media Types; in this case
/// the first Media Type returned is whatever is declared in the latest IETF RFC for the
/// presumed file format or the one that explicitly supersedes all others.
/// Ordering of additional Media Types is arbitrary.
///
/// ### Note: Values Not Stable
/// The exact Media Types returned in any given guess are not considered to be stable and are often
/// updated in patch releases in order to reflect the most up-to-date information possible.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
// FIXME: change repr when `mime` gains macro/const fn constructor
pub struct MimeGuess(&'static [&'static str]);

impl MimeGuess {
	/// Guess the MIME type of a file (real or otherwise) with the given extension.
	///
	/// The search is case-insensitive.
	///
	/// If `ext` is empty or has no (currently) known MIME type mapping, then an empty guess is
	/// returned.
	pub fn from_ext(ext: &str) -> Self {
		if ext.is_empty() {
			return Self(&[]);
		}

		impl_::get_mime_types(ext).map_or(Self(&[]), Self)
	}

	/// Guess the MIME type of `path` by its extension (as defined by
	/// [`Path::extension()`]). **No disk access is performed.**
	///
	/// If `path` has no extension, the extension cannot be converted to `str`, or has
	/// no known MIME type mapping, then an empty guess is returned.
	///
	/// The search is case-insensitive.
	///
	/// ## Note
	/// **Guess** is the operative word here, as there are no guarantees that the contents of the
	/// file that `path` points to match the MIME type associated with the path's extension.
	///
	/// Take care when processing files with assumptions based on the return value of this function.
	///
	/// [`Path::extension()`]: https://doc.rust-lang.org/std/path/struct.Path.html#method.extension
	pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
		path
			.as_ref()
			.extension()
			.and_then(OsStr::to_str)
			.map_or(Self(&[]), Self::from_ext)
	}

	/// `true` if the guess did not return any known mappings for the given path or extension.
	pub const fn is_empty(&self) -> bool { self.0.is_empty() }

	/// Get the number of MIME types in the current guess.
	pub const fn count(&self) -> usize { self.0.len() }

	/// Get the first guessed `Mime`, if applicable.
	///
	/// See [Note: Ordering](#note-ordering) above.
	pub fn first(&self) -> Option<Mime> { self.first_raw().map(expect_mime) }

	/// Get the first guessed Media Type as a string, if applicable.
	///
	/// See [Note: Ordering](#note-ordering) above.
	pub fn first_raw(&self) -> Option<&'static str> { self.0.get(0).copied() }

	/// Get the first guessed `Mime`, or if the guess is empty, return
	/// [`application/octet-stream`] instead.
	///
	/// See [Note: Ordering](#note-ordering) above.
	///
	/// ### Note: HTTP Applications
	/// For HTTP request and response bodies if a value for the `Content-Type` header
	/// cannot be determined it might be preferable to not send one at all instead of defaulting to
	/// `application/octet-stream` as the recipient will expect to infer the format directly from
	/// the content instead. ([RFC 7231, Section 3.1.1.5][rfc7231])
	///
	/// On the contrary, for `multipart/form-data` bodies, the `Content-Type` of a form-data part is
	/// assumed to be `text/plain` unless specified so a default of `application/octet-stream`
	/// for non-text parts is safer. ([RFC 7578, Section 4.4][rfc7578])
	///
	/// [`application/octet-stream`]: https://docs.rs/mime/0.3/mime/constant.APPLICATION_OCTET_STREAM.html
	/// [rfc7231]: https://tools.ietf.org/html/rfc7231#section-3.1.1.5
	/// [rfc7578]: https://tools.ietf.org/html/rfc7578#section-4.4
	pub fn first_or_octet_stream(&self) -> Mime { self.first_or(mime::APPLICATION_OCTET_STREAM) }

	/// Get the first guessed `Mime`, or if the guess is empty, return
	/// [`text/plain`](::mime::TEXT_PLAIN) instead.
	///
	/// See [Note: Ordering](#note-ordering) above.
	pub fn first_or_text_plain(&self) -> Mime { self.first_or(mime::TEXT_PLAIN) }

	/// Get the first guessed `Mime`, or if the guess is empty, return the given `Mime` instead.
	///
	/// See [Note: Ordering](#note-ordering) above.
	pub fn first_or(&self, default: Mime) -> Mime { self.first().unwrap_or(default) }

	/// Get the first guessed `Mime`, or if the guess is empty, execute the closure and return its
	/// result.
	///
	/// See [Note: Ordering](#note-ordering) above.
	pub fn first_or_else<F>(&self, default_fn: F) -> Mime
	where
		F: FnOnce() -> Mime,
	{
		self.first().unwrap_or_else(default_fn)
	}

	/// Get an iterator over the `Mime` values contained in this guess.
	///
	/// See [Note: Ordering](#note-ordering) above.
	pub fn iter(&self) -> Iter { Iter(self.iter_raw().map(expect_mime)) }

	/// Get an iterator over the raw media-type strings in this guess.
	///
	/// See [Note: Ordering](#note-ordering) above.
	pub fn iter_raw(&self) -> IterRaw { IterRaw(self.0.iter().copied()) }
}

impl IntoIterator for MimeGuess {
	type Item = Mime;
	type IntoIter = Iter;

	fn into_iter(self) -> Self::IntoIter { self.iter() }
}

impl<'a> IntoIterator for &'a MimeGuess {
	type Item = Mime;
	type IntoIter = Iter;

	fn into_iter(self) -> Self::IntoIter { self.iter() }
}

/// An iterator over the `Mime` types of a `MimeGuess`.
///
/// See [Note: Ordering on `MimeGuess`](struct.MimeGuess.html#note-ordering).
#[derive(Clone, Debug)]
pub struct Iter(iter::Map<IterRaw, fn(&'static str) -> Mime>);

impl Iterator for Iter {
	type Item = Mime;

	fn next(&mut self) -> Option<Self::Item> { self.0.next() }

	fn size_hint(&self) -> (usize, Option<usize>) { self.0.size_hint() }
}

impl DoubleEndedIterator for Iter {
	fn next_back(&mut self) -> Option<Self::Item> { self.0.next_back() }
}

impl FusedIterator for Iter {}

impl ExactSizeIterator for Iter {
	fn len(&self) -> usize { self.0.len() }
}

/// An iterator over the raw media type strings of a `MimeGuess`.
///
/// See [Note: Ordering on `MimeGuess`](struct.MimeGuess.html#note-ordering).
#[derive(Clone, Debug)]
pub struct IterRaw(iter::Copied<slice::Iter<'static, &'static str>>);

impl Iterator for IterRaw {
	type Item = &'static str;

	fn next(&mut self) -> Option<Self::Item> { self.0.next() }

	fn size_hint(&self) -> (usize, Option<usize>) { self.0.size_hint() }
}

impl DoubleEndedIterator for IterRaw {
	fn next_back(&mut self) -> Option<Self::Item> { self.0.next_back() }
}

impl FusedIterator for IterRaw {}

impl ExactSizeIterator for IterRaw {
	fn len(&self) -> usize { self.0.len() }
}

fn expect_mime(s: &str) -> Mime {
	// `.parse()` should be checked at compile time to never fail
	s.parse()
		.unwrap_or_else(|e| panic!("failed to parse media-type {:?}: {}", s, e))
}

/// Wrapper of [`MimeGuess::from_ext()`](struct.MimeGuess.html#method.from_ext).
pub fn from_ext(ext: &str) -> MimeGuess { MimeGuess::from_ext(ext) }

/// Wrapper of [`MimeGuess::from_path()`](struct.MimeGuess.html#method.from_path).
pub fn from_path<P: AsRef<Path>>(path: P) -> MimeGuess { MimeGuess::from_path(path) }

/// Get a list of known extensions for a given `Mime`.
///
/// Ignores parameters (only searches with `<main type>/<subtype>`). Case-insensitive (for extension types).
///
/// Returns `None` if the MIME type is unknown.
///
/// ### Wildcards
/// If the top-level of the MIME type is a wildcard (`*`), returns all extensions.
///
/// If the sub-level of the MIME type is a wildcard, returns all extensions for the top-level.
#[cfg(feature = "rev-mappings")]
pub fn get_mime_extensions(mime: &Mime) -> Option<&'static [&'static str]> {
	get_extensions(mime.type_().as_ref(), mime.subtype().as_ref())
}

/// Get a list of known extensions for a MIME type string.
///
/// Ignores parameters (only searches `<main type>/<subtype>`). Case-insensitive.
///
/// Returns `None` if the MIME type is unknown.
///
/// ### Wildcards
/// If the top-level of the MIME type is a wildcard (`*`), returns all extensions.
///
/// If the sub-level of the MIME type is a wildcard, returns all extensions for the top-level.
///
/// ### Panics
/// If `mime_str` is not a valid MIME type specifier (naive).
#[cfg(feature = "rev-mappings")]
pub fn get_mime_extensions_str(mut mime_str: &str) -> Option<&'static [&'static str]> {
	mime_str = mime_str.trim();

	if let Some(sep_idx) = mime_str.find(';') {
		mime_str = &mime_str[..sep_idx];
	}

	let (top, sub) = {
		let split_idx = mime_str.find('/')?;
		(&mime_str[..split_idx], &mime_str[split_idx + 1..])
	};

	get_extensions(top, sub)
}

/// Get the extensions for a given top-level and sub-level of a MIME type
/// (`{toplevel}/{sublevel}`).
///
/// Returns `None` if `toplevel` or `sublevel` are unknown.
///
/// ### Wildcards
/// If the top-level of the MIME type is a wildcard (`*`), returns all extensions.
///
/// If the sub-level of the MIME type is a wildcard, returns all extensions for the top-level.
#[cfg(feature = "rev-mappings")]
pub fn get_extensions(toplevel: &str, sublevel: &str) -> Option<&'static [&'static str]> {
	impl_::get_extensions(toplevel, sublevel)
}
