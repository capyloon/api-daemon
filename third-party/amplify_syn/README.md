# Derive helper library

![Build](https://github.com/rust-amplify/amplify-derive/workflows/Build/badge.svg)
![Tests](https://github.com/rust-amplify/amplify-derive/workflows/Tests/badge.svg)
![Lints](https://github.com/rust-amplify/amplify-derive/workflows/Lints/badge.svg)
[![codecov](https://codecov.io/gh/rust-amplify/amplify-derive/branch/master/graph/badge.svg)](https://codecov.io/gh/rust-amplify/rust-amplify)

[![crates.io](https://meritbadge.herokuapp.com/amplify_syn)](https://crates.io/crates/amplify_syn)
[![Docs](https://docs.rs/amplify_syn/badge.svg)](https://docs.rs/amplify_syn)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![Apache-2 licensed](https://img.shields.io/crates/l/amplify_syn)](./LICENSE)

Carefully crafted extensions to the well-known `syn` crate, which helps to
create complex derivation and proc macro libraries.

For samples, please check [documentation](https://docs.rs/amplify_syn) and 
the [following code](https://github.com/rust-amplify/amplify-derive/tree/master/src/getters.rs) 
from `amplify_derive` crate, which uses this library for its custom derivation 
macros.

Minimum supported rust compiler version (MSRV): 1.59.0. Rust edition 2021.
