// Copyright 2017 1aim GmbH
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
// of the Software, and to permit persons to whom the Software is furnished to do
// so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::ops::Deref;
use std::fmt;
use std::str;

use std::sync::Arc;
use oncemutex::OnceMutex;

use regex::{Regex, RegexBuilder, Error};
use crate::syntax;
use crate::options::Options;

/// A lazily created `Regex`.
///
/// At the first `Deref` the given source will be compiled and saved in the
/// Local Thread Storage, thus avoiding locking.
///
/// # Example
///
/// Find the location of a US phone number:
///
/// ```
/// # use regex_cache::LazyRegex;
/// let re = LazyRegex::new("[0-9]{3}-[0-9]{3}-[0-9]{4}").unwrap();
/// let m  = re.find("phone: 111-222-3333").unwrap();
/// assert_eq!((m.start(), m.end()), (7, 19));
/// ```
#[derive(Clone)]
pub struct LazyRegex {
	builder: LazyRegexBuilder,
	regex:   Arc<OnceMutex<Option<Regex>>>
}

impl LazyRegex {
	/// Create a new lazy `Regex` for the given source, checking the syntax is
	/// valid.
	pub fn new(source: &str) -> Result<LazyRegex, Error> {
		if let Err(err) = syntax::Parser::new().parse(source) {
			return Err(Error::Syntax(err.to_string()));
		}

		Ok(LazyRegex::from(LazyRegexBuilder::new(source)))
	}

	fn from(builder: LazyRegexBuilder) -> Self {
		LazyRegex {
			builder: builder,
			regex:   Arc::new(OnceMutex::new(None)),
		}
	}

	fn create(builder: &LazyRegexBuilder) -> Regex {
		builder.options.define(&mut RegexBuilder::new(&builder.source))
			.build().unwrap()
	}
}

impl Deref for LazyRegex {
	type Target = Regex;

	fn deref(&self) -> &Regex {
		self.as_ref()
	}
}

impl AsRef<Regex> for LazyRegex {
	fn as_ref(&self) -> &Regex {
		if let Some(mut guard) = self.regex.lock() {
			*guard = Some(LazyRegex::create(&self.builder));
		}

		(*self.regex).as_ref().unwrap()
	}
}

impl Into<Regex> for LazyRegex {
	fn into(self) -> Regex {
		let (regex, builder) = (self.regex, self.builder);

		Arc::try_unwrap(regex).ok().and_then(|m| m.into_inner()).unwrap_or_else(||
			LazyRegex::create(&builder))
	}
}

impl fmt::Debug for LazyRegex {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(&**self, f)
	}
}

impl fmt::Display for LazyRegex {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(&**self, f)
	}
}

impl str::FromStr for LazyRegex {
	type Err = Error;

	fn from_str(s: &str) -> Result<LazyRegex, Error> {
		LazyRegex::new(s)
	}
}

/// A configurable builder for a lazy `Regex`.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct LazyRegexBuilder {
	source: String,
	options: Options,
}

impl LazyRegexBuilder {
	/// Create a new regular expression builder with the given pattern.
	///
	/// If the pattern is invalid, then an error will be returned when
	/// `compile` is called.
	pub fn new(source: &str) -> LazyRegexBuilder {
		LazyRegexBuilder {
			source: source.to_owned(),
			options: Default::default(),
		}
	}

	/// Consume the builder and compile the regular expression.
	///
	/// Note that calling `as_str` on the resulting `Regex` will produce the
	/// pattern given to `new` verbatim. Notably, it will not incorporate any
	/// of the flags set on this builder.
	pub fn build(&self) -> Result<LazyRegex, Error> {
		if let Err(err) = syntax::Parser::new().parse(&self.source) {
			return Err(Error::Syntax(err.to_string()));
		}

		Ok(LazyRegex::from(self.clone()))
	}

	/// Set the value for the case insensitive (`i`) flag.
	pub fn case_insensitive(&mut self, yes: bool) -> &mut LazyRegexBuilder {
		self.options.case_insensitive = yes;
		self
	}

	/// Set the value for the multi-line matching (`m`) flag.
	pub fn multi_line(&mut self, yes: bool) -> &mut LazyRegexBuilder {
		self.options.multi_line = yes;
		self
	}

	/// Set the value for the any character (`s`) flag, where in `.` matches
	/// anything when `s` is set and matches anything except for new line when
	/// it is not set (the default).
	///
	/// N.B. "matches anything" means "any byte" for `regex::bytes::Regex`
	/// expressions and means "any Unicode scalar value" for `regex::Regex`
	/// expressions.
	pub fn dot_matches_new_line(&mut self, yes: bool) -> &mut LazyRegexBuilder {
		self.options.dot_matches_new_line = yes;
		self
	}

	/// Set the value for the greedy swap (`U`) flag.
	pub fn swap_greed(&mut self, yes: bool) -> &mut LazyRegexBuilder {
		self.options.swap_greed = yes;
		self
	}

	/// Set the value for the ignore whitespace (`x`) flag.
	pub fn ignore_whitespace(&mut self, yes: bool) -> &mut LazyRegexBuilder {
		self.options.ignore_whitespace = yes;
		self
	}

	/// Set the value for the Unicode (`u`) flag.
	pub fn unicode(&mut self, yes: bool) -> &mut LazyRegexBuilder {
		self.options.unicode = yes;
		self
	}

	/// Set the approximate size limit of the compiled regular expression.
	///
	/// This roughly corresponds to the number of bytes occupied by a single
	/// compiled program. If the program exceeds this number, then a
	/// compilation error is returned.
	pub fn size_limit(&mut self, limit: usize) -> &mut LazyRegexBuilder {
		self.options.size_limit = limit;
		self
	}

	/// Set the approximate size of the cache used by the DFA.
	///
	/// This roughly corresponds to the number of bytes that the DFA will
	/// use while searching.
	///
	/// Note that this is a *per thread* limit. There is no way to set a global
	/// limit. In particular, if a regex is used from multiple threads
	/// simulanteously, then each thread may use up to the number of bytes
	/// specified here.
	pub fn dfa_size_limit(&mut self, limit: usize) -> &mut LazyRegexBuilder {
		self.options.dfa_size_limit = limit;
		self
	}
}

#[cfg(test)]
mod test {
	use crate::{LazyRegex, LazyRegexBuilder};

	#[test]
	fn new() {
		assert!(LazyRegex::new(r"^\d+$").unwrap()
			.is_match("2345"));

		assert!(!LazyRegex::new(r"^[a-z]+$").unwrap()
			.is_match("2345"));
	}

	#[test]
	fn build() {
		assert!(LazyRegexBuilder::new(r"^abc$")
			.case_insensitive(true).build().unwrap()
			.is_match("ABC"));

		assert!(!LazyRegexBuilder::new(r"^abc$")
			.case_insensitive(false).build().unwrap()
			.is_match("ABC"));
	}

	#[test]
	fn same() {
		let re = LazyRegex::new(r"^\d+$").unwrap();

		assert!(re.is_match("1234"));
		assert!(re.is_match("1234"));
		assert!(re.is_match("1234"));
		assert!(re.is_match("1234"));
	}
}
