//! Provide a lowercased diacritics-free version of a character or a string.
//!
//! For example return `e` for `é`.
//!
//! Secular's char lookup is an inlined lookup of a static table, which means it's possible to use it in performance sensitive code.
//!
//! Secular also performs (optionally) Unicode normalization.
//!
//! ## Declaration
//!
//! By default, diacritics removal is only done on ascii chars, so to include a smaller table.
//!
//! If you want to handle the whole BMP, use the "bmp" feature" (the binary will be bigger to
//! incorporate the whole mapping).
//!
//! Default import:
//!
//!```toml
//! [dependencies]
//! secular = "0.3"
//! ```
//!
//! For more characters (the BMP):
//!
//!```toml
//![dependencies]
//!secular = { version="0.3", features=["bmp"] }
//! ```
//!
//! With Unicode normalization functions (using the unicode-normalization crate):
//!
//!```toml
//![dependencies]
//!secular = { version="0.3", features=["normalization"] }
//! ```
//!
//! or
//!
//!```toml
//![dependencies]
//!secular = { version="0.3", features=["bmp","normalization"] }
//! ```
//!
//! This feature is optional so that you can avoid importing the unicode-normalization crate (note that it's used in many other crates so it's possible your text processing application already uses it).
//!
//! ## Usage
//!
//! On characters:
//!
//! ```
//! use secular::*;
//! let s = "Comunicações"; // normalized string (length=12)
//! let chars: Vec<char> = s.chars().collect();
//! assert_eq!(chars.len(), 12);
//! assert_eq!(chars[0], 'C');
//! assert_eq!(lower_lay_char(chars[0]), 'c');
//! assert_eq!(chars[8], 'ç');
//! assert_eq!(lower_lay_char(chars[8]), 'c');
//! ```
//!
//! On strings:
//!
//! ```
//! use secular::*;
//! let s = "Comunicações"; // unnormalized string (length=14)
//! assert_eq!(s.chars().count(), 14);
//! let s = normalized_lower_lay_string(s);
//! assert_eq!(s.chars().count(), 12);
//! assert_eq!(s, "comunicacoes");
//! ```

#[cfg(not(feature = "bmp"))]
mod data_ascii;
#[cfg(not(feature = "bmp"))]
use data_ascii::LAY_CHARS;

#[cfg(feature = "bmp")]
mod data_bmp;
#[cfg(feature = "bmp")]
use data_bmp::LAY_CHARS;

#[cfg(feature = "normalization")]
use unicode_normalization::{
    UnicodeNormalization,
};

/// Return a lowercased diacritics-free version of the character.
///
/// If the character is outside of the ASCII range and the "bmp"
/// feature wasn't included, return the same character, unchanged.
#[inline(always)]
pub fn lower_lay_char(c: char) -> char {
    // this is functionally the same than
    //      LAY_CHARS.get(c as usize).copied().unwrap_or(c)
    // but much faster
    if (c as usize) < LAY_CHARS.len() {
        unsafe {
            *LAY_CHARS.get_unchecked(c as usize)
        }
    } else {
        c
    }
}

/// Replace every character with its lowercased diacritics-free equivalent
/// whenever possible.
///
/// By construct, the resulting string is guaranteed to have the same number
/// of characters as the input one (it may be smaller in bytes but not larger).
///
/// This function doesn't do any normalization. It's thus necessary to ensure
/// the string is already normalized.
pub fn lower_lay_string(s: &str) -> String {
    s.chars()
        .map(|c| lower_lay_char(c))
        .collect()
}

/// Normalize the string then replace every character with its
/// lowercased diacritics-free equivalent whenever possible.
#[cfg(feature = "normalization")]
pub fn normalized_lower_lay_string(s: &str) -> String {
    s.nfc()
        .map(|c| lower_lay_char(c))
        .collect()
}


// To test, run
//     cargo test --features="bmp, normalization"
// or
//     bacon test
#[cfg(all(test, feature="normalization"))]
mod tests {
    use super::*;
    #[test]
    fn test_lower_lay_char() {
        let s = "Comunicações"; // normalized string (length=12 characters)
        let chars: Vec<char> = s.chars().collect();
        assert_eq!(chars.len(), 12);
        assert_eq!(chars[0], 'C');
        assert_eq!(lower_lay_char(chars[0]), 'c');
        assert_eq!(chars[8], 'ç');
        assert_eq!(lower_lay_char(chars[8]), 'c');
    }
    #[test]
    fn test_normalized_lower_lay_string() {
        let s = "Comunicações"; // unnormalized string (length=14 characters)
        assert_eq!(s.chars().count(), 14);
        let s = normalized_lower_lay_string(s);
        assert_eq!(s.chars().count(), 12);
        assert_eq!(s, "comunicacoes");
    }
}

