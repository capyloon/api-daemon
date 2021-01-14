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

use std::ops::{Deref, DerefMut};
use std::sync::{Mutex, Arc};
use std::borrow::Cow;
use std::fmt;
use std::str;

use regex::{Regex, RegexBuilder, Error};
use regex::{Match, Captures, Replacer};
use crate::syntax;
use crate::options::Options;
use crate::lru::LruCache;

/// An LRU cache for regular expressions.
#[derive(Clone, Debug)]
pub struct RegexCache(LruCache<String, Regex>);

impl RegexCache {
	/// Create a new LRU cache with the given size limit.
	pub fn new(capacity: usize) -> RegexCache {
		RegexCache(LruCache::new(capacity))
	}

	/// Save the given regular expression in the cache.
	///
	/// # Example
	///
	/// ```
	/// # use regex_cache::{Regex, RegexCache};
	/// let mut cache = RegexCache::new(100);
	/// let     re    = Regex::new(r"^\d+$").unwrap();
	///
	/// // By saving the previously created regular expression further calls to
	/// // `compile` won't actually compile the regular expression.
	/// cache.save(re);
	///
	/// assert!(cache.compile(r"^\d+$").unwrap().is_match("1234"));
	/// assert!(!cache.compile(r"^\d+$").unwrap().is_match("abcd"));
	/// ```
	pub fn save(&mut self, re: Regex) -> &Regex {
		let source = re.as_str().to_owned();

		if !self.0.contains_key(re.as_str()) {
			self.insert(source.clone(), re);
		}

		self.0.get_mut(&source).unwrap()
	}

	/// Create a new regular expression in the cache.
	///
	/// # Example
	///
	/// ```
	/// # use regex_cache::RegexCache;
	/// let mut cache = RegexCache::new(100);
	///
	/// assert!(cache.compile(r"^\d+$").unwrap().is_match("1234"));
	/// assert!(!cache.compile(r"^\d+$").unwrap().is_match("abcd"));
	/// ```
	pub fn compile(&mut self, source: &str) -> Result<&Regex, Error> {
		if !self.0.contains_key(source) {
			self.0.insert(source.into(), Regex::new(source)?);
		}

		Ok(self.0.get_mut(source).unwrap())
	}

	/// Configure a new regular expression.
	///
	/// # Example
	///
	/// ```
	/// # use regex_cache::RegexCache;
	/// let mut cache = RegexCache::new(100);
	///
	/// assert!(cache.configure(r"abc", |b| b.case_insensitive(true)).unwrap()
	/// 	.is_match("ABC"));
	///
	/// assert!(!cache.configure(r"abc", |b| b.case_insensitive(true)).unwrap()
	/// 	.is_match("123"));
	/// ```
	pub fn configure<F>(&mut self, source: &str, f: F) -> Result<&Regex, Error>
		where F: FnOnce(&mut RegexBuilder) -> &mut RegexBuilder
	{
		if !self.0.contains_key(source) {
			self.0.insert(source.into(), f(&mut RegexBuilder::new(source)).build()?);
		}

		Ok(self.0.get_mut(source).unwrap())
	}
}

impl Deref for RegexCache {
	type Target = LruCache<String, Regex>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for RegexCache {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

#[derive(Clone)]
pub struct CachedRegex {
	builder: CachedRegexBuilder,
}

macro_rules! regex {
	($self:ident) => (
		$self.builder.cache.lock().unwrap().configure(&$self.builder.source, |b|
			$self.builder.options.define(b)).unwrap()
	)
}

impl CachedRegex {
	/// Create a new cached `Regex` for the given source, checking the syntax is
	/// valid.
	pub fn new(cache: Arc<Mutex<RegexCache>>, source: &str) -> Result<CachedRegex, Error> {
		if let Err(err) = syntax::Parser::new().parse(source) {
			return Err(Error::Syntax(err.to_string()));
		}

		Ok(CachedRegex::new_unchecked(cache, source))
	}

	/// Create a new cached `Regex` for the given source, without checking if the 
	/// syntax is valid.
	/// 
	/// Only use this if you know that the syntax is valid or you are ready to 
	/// handle potential syntax errors later on.
	pub fn new_unchecked(cache: Arc<Mutex<RegexCache>>, source: &str) -> CachedRegex {
		CachedRegex::from(CachedRegexBuilder::new(cache, source))
	}

	fn from(builder: CachedRegexBuilder) -> Self {
		CachedRegex {
			builder: builder,
		}
	}

	/// Refer to `Regex::is_match`.
	pub fn is_match(&self, text: &str) -> bool {
		regex!(self).is_match(text)
	}

	/// Refer to `Regex::find`.
	pub fn find<'t>(&self, text: &'t str) -> Option<Match<'t>> {
		regex!(self).find(text)
	}

	/// Refer to `Regex::captures`.
	pub fn captures<'t>(&self, text: &'t str) -> Option<Captures<'t>> {
		regex!(self).captures(text)
	}

	/// Refer to `Regex::replace`.
	pub fn replace<'t, R: Replacer>(&self, text: &'t str, rep: R) -> Cow<'t, str> {
		regex!(self).replace(text, rep)
	}

	/// Refer to `Regex::replace_all`.
	pub fn replace_all<'t, R: Replacer>(&self, text: &'t str, rep: R) -> Cow<'t, str> {
		regex!(self).replace_all(text, rep)
	}

	/// Refer to `Regex::shortest_match`.
	pub fn shortest_match(&self, text: &str) -> Option<usize> {
		regex!(self).shortest_match(text)
	}

	pub fn captures_len(&self) -> usize {
		regex!(self).captures_len()
	}

	pub fn as_str(&self) -> &str {
		&self.builder.source
	}
}

impl fmt::Debug for CachedRegex {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(regex!(self), f)
	}
}

impl fmt::Display for CachedRegex {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(regex!(self), f)
	}
}

/// A configurable builder for a cached `Regex`.
#[derive(Clone, Debug)]
pub struct CachedRegexBuilder {
	cache:   Arc<Mutex<RegexCache>>,
	source:  String,
	options: Options,
}

impl CachedRegexBuilder {
	/// Create a new regular expression builder with the given pattern.
	///
	/// If the pattern is invalid, then an error will be returned when
	/// `compile` is called.
	pub fn new(cache: Arc<Mutex<RegexCache>>, source: &str) -> CachedRegexBuilder {
		CachedRegexBuilder {
			cache:   cache,
			source:  source.to_owned(),
			options: Default::default(),
		}
	}

	/// Consume the builder and compile the regular expression.
	///
	/// Note that calling `as_str` on the resulting `Regex` will produce the
	/// pattern given to `new` verbatim. Notably, it will not incorporate any
	/// of the flags set on this builder.
	pub fn build(&self) -> Result<CachedRegex, Error> {
		if let Err(err) = syntax::Parser::new().parse(&self.source) {
			return Err(Error::Syntax(err.to_string()));
		}

		Ok(CachedRegex::from(self.clone()))
	}

	/// Consume the builder and compile the regular expression without checking 
	/// if the syntax is valid.
	/// 
	/// Only use this if you know that the syntax is valid or you are ready to 
	/// handle potential syntax errors later on.
	///
	/// Note that calling `as_str` on the resulting `Regex` will produce the
	/// pattern given to `new` verbatim. Notably, it will not incorporate any
	/// of the flags set on this builder.
	pub fn build_unchecked(&self) -> CachedRegex {
		CachedRegex::from(self.clone())
	}

	/// Set the value for the case insensitive (`i`) flag.
	pub fn case_insensitive(&mut self, yes: bool) -> &mut CachedRegexBuilder {
		self.options.case_insensitive = yes;
		self
	}

	/// Set the value for the multi-line matching (`m`) flag.
	pub fn multi_line(&mut self, yes: bool) -> &mut CachedRegexBuilder {
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
	pub fn dot_matches_new_line(&mut self, yes: bool) -> &mut CachedRegexBuilder {
		self.options.dot_matches_new_line = yes;
		self
	}

	/// Set the value for the greedy swap (`U`) flag.
	pub fn swap_greed(&mut self, yes: bool) -> &mut CachedRegexBuilder {
		self.options.swap_greed = yes;
		self
	}

	/// Set the value for the ignore whitespace (`x`) flag.
	pub fn ignore_whitespace(&mut self, yes: bool) -> &mut CachedRegexBuilder {
		self.options.ignore_whitespace = yes;
		self
	}

	/// Set the value for the Unicode (`u`) flag.
	pub fn unicode(&mut self, yes: bool) -> &mut CachedRegexBuilder {
		self.options.unicode = yes;
		self
	}

	/// Set the approximate size limit of the compiled regular expression.
	///
	/// This roughly corresponds to the number of bytes occupied by a single
	/// compiled program. If the program exceeds this number, then a
	/// compilation error is returned.
	pub fn size_limit(&mut self, limit: usize) -> &mut CachedRegexBuilder {
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
	pub fn dfa_size_limit(&mut self, limit: usize) -> &mut CachedRegexBuilder {
		self.options.dfa_size_limit = limit;
		self
	}
}

#[cfg(test)]
mod test {
	use std::sync::{Arc, Mutex};
	use crate::cache::{RegexCache, CachedRegex};

	#[test]
	fn respects_limit() {
		let mut cache = RegexCache::new(2);

		cache.compile("[01]2").unwrap();
		cache.compile("[21]0").unwrap();

		assert_eq!(cache.len(), 2);
		cache.compile("[21]3").unwrap();
		assert_eq!(cache.len(), 2);
	}

	#[test]
	fn cached_regex() {
		let cache = Arc::new(Mutex::new(RegexCache::new(100)));
		let re = CachedRegex::new(cache.clone(), r"^\d+$").unwrap();

		assert!(re.is_match("123"));
		assert!(!re.is_match("abc"));
	}
}
