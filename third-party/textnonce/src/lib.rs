// Copyright 2015-2020 textnonce Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! A nonce is a cryptographic concept of an arbitrary number that is never used
//! more than once.
//!
//! `TextNonce` is a nonce because the first 16 characters represents the current
//! time, which will never have been generated before, nor will it be generated
//! again, across the period of time in which a `Timespec` (or `std::time::Duration`,
//! counting from the UNIX epoch) is valid.
//!
//! `TextNonce` additionally includes bytes of randomness, making it difficult to
//! predict. This makes it suitable to be used for a session ID.
//!
//! It is also text-based, using only characters in the base64 character set.
//!
//! Various length `TextNonce`es may be generated.  The minimum length is 16
//! characters, and lengths must be evenly divisible by 4.

#![deny(missing_debug_implementations, trivial_casts, trivial_numeric_casts,
        unused_import_braces, unused_qualifications, unused_results, unused_lifetimes,
        unused_labels, unused_extern_crates, non_ascii_idents, keyword_idents,
        deprecated_in_future, unstable_features, single_use_lifetimes, unsafe_code,
        unreachable_pub, missing_docs, missing_copy_implementations)]

use rand::rngs::OsRng;
use rand::RngCore;
use std::fmt;
use std::io::Cursor;
use std::ops::Deref;
use std::io::Write;

/// A nonce is a cryptographic concept of an arbitrary number that is never used
/// more than once.
///
/// `TextNonce` is a nonce because the first 16 characters represents the current
/// time, which will never have been generated before, nor will it be generated
/// again, across the period of time in which a `Timespec` (or `std::time::Duration`,
/// counting from the UNIX epoch) is valid.
///
/// `TextNonce` additionally includes bytes of randomness, making it difficult to
/// predict. This makes it suitable to be used for session IDs.
///
/// It is also text-based, using only characters in the base64 character set.
///
/// Various length `TextNonce`es may be generated.  The minimum length is 16
/// characters, and lengths must be evenly divisible by 4.
#[derive(Clone, PartialEq, Debug, Default)]
#[cfg_attr(feature ="serde", derive(Serialize, Deserialize))]
pub struct TextNonce(pub String);

impl TextNonce {
    /// Generate a new `TextNonce` with 16 characters of time and 16 characters of
    /// randomness
    pub fn new() -> TextNonce {
        TextNonce::sized(32).unwrap()
    }

    /// Generate a new `TextNonce`. `length` must be at least 16, and divisible by 4.
    /// The first 16 characters come from the time component, and all characters
    /// after that will be random.
    pub fn sized(length: usize) -> Result<TextNonce, String> {
        TextNonce::sized_configured(length, base64::STANDARD)
    }

    /// Generate a new `TextNonce` using the `URL_SAFE` variant of base64 (using '_' and '-')
    /// `length` must be at least 16, and divisible by 4.  The first 16 characters come
    /// from the time component, and all characters after that will be random.
    pub fn sized_urlsafe(length: usize) -> Result<TextNonce, String> {
        TextNonce::sized_configured(length, base64::URL_SAFE)
    }

    /// Generate a new `TextNonce` specifying the Base64 configuration to use.
    /// `length` must be at least 16, and divisible by 4.  The first 16 characters come
    /// from the time component, and all characters after that will be random.
    pub fn sized_configured(length: usize, config: base64::Config) -> Result<TextNonce, String> {
        if length < 16 {
            return Err("length must be >= 16".to_owned());
        }
        if length % 4 != 0 {
            return Err("length must be divisible by 4".to_owned());
        }

        let bytelength: usize = (length / 4) * 3;

        let mut raw: Vec<u8> = vec![0; bytelength];

        // Get the first 12 bytes from the current time
        {
            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
                .map_err(|_| "creating nonces from before UNIX epoch not supported".to_string())?;
            let secs: u64 = now.as_secs();
            let nsecs: u32 = now.subsec_nanos();

            let mut cursor = Cursor::new(&mut *raw);
            cursor.write_all(&nsecs.to_le_bytes()).unwrap();
            cursor.write_all(&secs.to_le_bytes()).unwrap();
        }

        // Get the last bytes from random data

        OsRng.fill_bytes(&mut raw[12..bytelength]);

        // base64 encode
        Ok(TextNonce(base64::encode_config(&raw, config)))
    }

    /// Convert into a string
    pub fn into_string(self) -> String {
        let TextNonce(s) = self;
        s
    }
}

impl fmt::Display for TextNonce {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl Deref for TextNonce {
    type Target = str;
    fn deref(&self) -> &str {
        &*self.0
    }
}

#[cfg(test)]
mod tests {
    use super::TextNonce;
    use std::collections::HashSet;

    #[test]
    fn new() {
        // Test 100 nonces:
        let mut map = HashSet::new();
        for _ in 0..100 {
            let n = TextNonce::new();
            let TextNonce(s) = n;

            // Verify their length
            assert_eq!(s.len(), 32);

            // Verify their character content
            assert_eq!(
                s.chars()
                    .filter(|x| x.is_digit(10) || x.is_alphabetic() || *x == '+' || *x == '/')
                    .count(),
                32
            );

            // Add to the map
            let _ = map.insert(s);
        }
        assert_eq!(map.len(), 100);
    }

    #[test]
    fn sized() {
        let n = TextNonce::sized(48);
        assert!(n.is_ok());
        let TextNonce(s) = n.unwrap();
        assert_eq!(s.len(), 48);

        let n = TextNonce::sized(47);
        assert!(n.is_err());
        let n = TextNonce::sized(12);
        assert!(n.is_err());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde() {
        use bincode;
        use serde::{Serialize, Deserialize};

        let n = TextNonce::sized(48);
        let serialized = bincode::serialize(&n).unwrap();
        let deserialized = bincode::deserialize(&serialized).unwrap();
        assert_eq!(n, deserialized);
    }
}
