use std::fmt::Debug;
use std::path::Path;

use super::{expect_mime, from_ext, from_path, get_mime_extensions_str};
use crate::mime_types::MIME_TYPES;

#[test]
fn check_type_bounds() {
	fn assert_type_bounds<T: Clone + Debug + Send + Sync + 'static>() {}

	assert_type_bounds::<super::MimeGuess>();
	assert_type_bounds::<super::Iter>();
	assert_type_bounds::<super::IterRaw>();
}

#[test]
/// Test guessing MIME type based on file paths and extensions.
fn test_mime_type_guessing() {
	assert_eq!(from_ext("gif").first_or_octet_stream().to_string(), "image/gif".to_string());
	assert_eq!(from_ext("TXT").first_or_octet_stream().to_string(), "text/plain".to_string());
	assert_eq!(
		from_ext("blahblah").first_or_octet_stream().to_string(),
		"application/octet-stream".to_string()
	);

	assert_eq!(
		from_path(Path::new("/path/to/file.gif"))
			.first_or_octet_stream()
			.to_string(),
		"image/gif".to_string()
	);
	assert_eq!(
		from_path("/path/to/file.gif").first_or_octet_stream().to_string(),
		"image/gif".to_string()
	);
}

#[test]
/// Test that guessing correctly returns the expected `Option`s.
fn test_mime_type_guessing_opt() {
	assert_eq!(from_ext("gif").first().unwrap().to_string(), "image/gif".to_string());
	assert_eq!(from_ext("TXT").first().unwrap().to_string(), "text/plain".to_string());
	assert_eq!(from_ext("blahblah").first(), None);

	assert_eq!(
		from_path("/path/to/file.gif").first().unwrap().to_string(),
		"image/gif".to_string()
	);
	assert_eq!(from_path("/path/to/file").first(), None);
}

#[test]
/// Ensures that each MIME type listed in MIME_TYPES is valid.
fn test_are_mime_types_parseable() {
	for (_, mimes) in MIME_TYPES {
		mimes.iter().for_each(|s| {
			expect_mime(s);
		});
	}
}

// RFC: Is this test necessary anymore? --@cybergeek94, 2/1/2016
#[test]
/// Ensures that all given file extensions are valid ASCII.
fn test_are_extensions_ascii() {
	for (ext, _) in MIME_TYPES {
		assert!(ext.is_ascii(), "Extension not ASCII: {:?}", ext);
	}
}

#[test]
/// Ensures that extensions are sorted.
fn test_are_extensions_sorted() {
	// simultaneously checks the requirement that duplicate extension entries are adjacent
	for (&(ext, _), &(n_ext, _)) in MIME_TYPES.iter().zip(MIME_TYPES.iter().skip(1)) {
		assert!(
			ext <= n_ext,
			"Extensions in src/mime_types should be sorted lexicographically
                in ascending order. Failed assert: {:?} <= {:?}",
			ext,
			n_ext
		);
	}
}

#[test]
/// Ensures that [`get_mime_extensions_str`] does not panic when given an invalid MIME type.
fn test_get_mime_extensions_str_no_panic_if_bad_mime() {
	assert_eq!(get_mime_extensions_str(""), None);
}

#[test]
/// Ensures that there are no duplicate extensions listed in [`MIME_TYPES`].
fn no_duplicate_mime_types() {
	let mut exts = Vec::with_capacity(MIME_TYPES.len());
	for (ext, _) in MIME_TYPES.into_iter() {
		assert!(
			!exts.contains(ext),
			"Duplicate extension found: {} appears more than once in MIME_TYPES.",
			ext
		);
		exts.push(ext);
	}
}
