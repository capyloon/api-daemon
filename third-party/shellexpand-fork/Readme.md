shellexpand, a library for shell-like expansion in strings
==========================================================

[![Build Status][actions]](https://github.com/netvl/shellexpand/actions?query=workflow%3ACI)
[![crates.io][crates]](https://crates.io/crates/shellexpand)
[![docs][docs]](https://docs.rs/shellexpand)

  [actions]: https://img.shields.io/github/workflow/status/netvl/shellexpand/CI/master?style=flat-square
  [crates]: https://img.shields.io/crates/v/shellexpand.svg?style=flat-square
  [docs]: https://img.shields.io/badge/docs-latest%20release-6495ed.svg?style=flat-square

[Documentation](https://docs.rs/shellexpand/)

shellexpand is a single dependency library which allows one to perform shell-like expansions in strings,
that is, to expand variables like `$A` or `${B}` into their values inside some context and to expand
`~` in the beginning of a string into the home directory (again, inside some context).

This crate provides generic functions which accept arbitrary contexts as well as default, system-based
functions which perform expansions using the system-wide context (represented by functions from `std::env`
module and [dirs-next](https://crates.io/crates/dirs-next) crate).

---

**Note: This package is a fork of [Vladimir Matveev's library](https://crates.io/crates/shellexpand),
which is no longer maintained.**

---

## Usage

Add a dependency in your `Cargo.toml`.

To avoid having to write `use shellexpand_fork as shellexpand` or similar,
rename the package on import:

```toml
[dependencies]
shellexpand = { version = "2.1", package = "shellexpand-fork" }o
```

See the crate documentation (a link is present in the beginning of this readme) for more information
and examples.


## Changelog

### Version 2.1.1

* Fix tilde expanding on Windows with Windows style (backslash) paths.
  Addresses <https://github.com/netvl/shellexpand/pull/13>.
* Forked as `shellexpand-fork` on crates.io.

### Version 2.1.0

* Switched to `dirs-next` instead of the obsolete `dirs` as the underlying dependency used to resolve the home directory
* Switched to GitHub Actions instead of Travis CI for building the project.

### Version 2.0.0

* Added support for default values in variable expansion (i.e. `${ANSWER:-42}`)
* Breaking changes (minimum Rust version is now 1.30.0):
  + Using `dyn` for trait objects to fix deprecation warning
  + Switched to using `source()` instead of `cause()` in the `Error` implementation, and
    therefore added a `'static` bound for the generic error parameter `E`

### Version 1.1.1

* Bump `dirs` dependency to 2.0.

### Version 1.1.0

* Changed use of deprecated `std::env::home_dir` to the [dirs](https://crates.io/crates/dirs)::home_dir function

### Version 1.0.0

* Fixed typos and minor incompletenesses in the documentation
* Changed `home_dir` argument type for tilde expansion functions to `FnOnce` instead `FnMut`
* Changed `LookupError::name` field name to `var_name`

### Version 0.1.0

* Initial release

## License

This program is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed 
as above, without any additional terms or conditions.
