![Build](https://github.com/ijackson/rust-derive-builder/workflows/Build/badge.svg?branch=master)
[![Documentation](https://docs.rs/derive_builder_core_fork_arti/badge.svg)](https://docs.rs/derive_builder_core_fork_arti)
[![Latest version](https://img.shields.io/crates/v/derive_builder_core_fork_arti.svg)](https://crates.io/crates/derive_builder_core_fork_arti)
[![All downloads](https://img.shields.io/crates/d/derive_builder_core_fork_arti.svg)](https://crates.io/crates/derive_builder_core_fork_arti)
[![Downloads of latest version](https://img.shields.io/crates/dv/derive_builder_core_fork_arti.svg)](https://crates.io/crates/derive_builder_core_fork_arti)

# Crate [`derive_builder_core`]

**Important Note**:

* You are probably looking for the [`derive_builder`] crate,
  which wraps this crate and is much more ergonomic to use.
* The API of this crate might **change frequently** in the near future.
  The [`derive_builder`] crate also provides a much more stable API.

## **THIS IS A FORK**

This version of `derive_builder` has an additional `sub_builder` feature,
which [has not been accepted upstream](https://github.com/colin-kiegel/rust-derive-builder/issues/254).
We may add further additional features.

In other respects, this fork is likely to lag behind the upstream crate.

## Purpose

This is an internal helper library of [`derive_builder`]. Its purpose is to
allow [`derive_builder`] to use its own code generation technique, if needed.

[`derive_builder_core`] might also be used in crates that [`derive_builder`]
depends on - to break a dependency cycle.

If [`derive_builder`] does not itself depend on _your_ crate, then you
should consider using [`derive_builder`] instead of [`derive_builder_core`].

[`derive_builder`]: https://crates.io/crates/derive_builder
[`derive_builder_core`]: https://crates.io/crates/derive_builder_core

## Documentation

Please refer to
[docs.rs/derive_builder_core](https://docs.rs/derive_builder_core)
for the documentation of all published versions.

## [Changelog](CHANGELOG.md)

Yes, we keep a changelog.

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
