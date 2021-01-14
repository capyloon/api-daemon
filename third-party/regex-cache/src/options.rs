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

use regex::RegexBuilder;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Options {
	pub case_insensitive: bool,
	pub multi_line: bool,
	pub dot_matches_new_line: bool,
	pub swap_greed: bool,
	pub ignore_whitespace: bool,
	pub unicode: bool,
	pub size_limit: usize,
	pub dfa_size_limit: usize,
}

impl Default for Options {
	fn default() -> Self {
		Options {
			case_insensitive: false,
			multi_line: false,
			dot_matches_new_line: false,
			swap_greed: false,
			ignore_whitespace: false,
			unicode: true,
			size_limit: 10 * (1 << 20),
			dfa_size_limit: 2 * (1 << 20),
		}
	}
}

impl Options {
	pub fn define<'b>(&self, builder: &'b mut RegexBuilder) -> &'b mut RegexBuilder {
		builder
			.case_insensitive(self.case_insensitive)
			.multi_line(self.multi_line)
			.dot_matches_new_line(self.dot_matches_new_line)
			.swap_greed(self.swap_greed)
			.ignore_whitespace(self.ignore_whitespace)
			.unicode(self.unicode)
			.size_limit(self.size_limit)
			.dfa_size_limit(self.dfa_size_limit)
	}
}
