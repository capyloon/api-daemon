# Changelog
Dates are given in YYYY-MM-DD format - for example, the 15th of October 2021 is written as 2021-10-15.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 4.0.0 - 2021-11-25
### Changed
- **Breaking:** Increased MSRV to 1.48.0
- Made some functions const
- Updated all dependencies!

## 3.0.2 - 2021-11-25
### Added
- This changelog :L
- Test for duplicate entries in MIME_TYPES
### Fixed
- Some formats had incorrect MIME types ([#3])
### Removed
- Removed a few duplicate entries from `MIME_TYPES` ([#3])

## 3.0.1 - 2021-09-24
### Added
- Clippy config (`clippy.toml`)
### Changed
- Update `phf` and `phf_codegen` dependencies
### Removed
- `Cargo.lock` is no longer committed

## 3.0.0 - 2021-08-06
### Added
- Surprisingly enough, more formats, including many from upstream sources
- The previously disabled `phf-map` feature has been fixed and can now be used by projects depending on this library 
  by setting the appropriate feature flag
- The MSRV is now 1.40.0
- Updated to Rust 2018 Edition
### Changed
- Renamed from `mime_guess` to `new_mime_guess`
- Reformatted all files and added `rustfmt.toml`
### Removed
- `mime` and `unicase` are no longer `extern crate`d
- All functions marked as deprecated from the 2.x releases have been removed

## 2.1.1 - 2021-07-21
### Added
- New formats, including Cucumber/Gherkin related files, and a few linked data formats
### Changed
- New mime types for existing formats:
  - `application/x-gzip` -> `application/gzip`
  - `application/content-stream` -> `application/octet-stream`

## 2.1.0 - 2021-04-28
### Added
- Many programming-adjacent file types, such as `bash` and `php`
- Committed `Cargo.lock` file

## 2.0.4 - 2021-04-28
### Added
- Many file types, including `exe`, `dll`, and `scr` files
- Added some more MIME types for existing formats
### Changed
- Renamed project to `new_mime_guess`
### Fixed
- Potential XML and HTML mislabeling

## Older
Older releases are not tracked in this project's changelog, as they are from before it was forked from its [parent 
project](https://github.com/abonander/mime_guess)

<!-- links -->
[#3]: https://github.com/Lynnesbian/new_mime_guess/pull/3
