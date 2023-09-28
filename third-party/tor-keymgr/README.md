# tor-keymgr

Code to fetch, store, and update keys.

## Overview

This crate is part of
[Arti](https://gitlab.torproject.org/tpo/core/arti/), a project to
implement [Tor](https://www.torproject.org/) in Rust.

### **Likely to change**

The APIs exposed by this crate (even without the `keymgr` feature)
are new and are likely to change rapidly.
We'll therefore often be making semver-breaking changes
(and will update the crate version accordingly).

## Key stores

The [`KeyMgr`] is an interface to one or more key stores. The key
stores are types that implement the [`Keystore`] trait.

This crate provides the following key store implementations:
* Arti key store: an on-disk store that stores keys in OpenSSH format.
* (not yet implemented) C Tor key store: an on-disk store that is
  backwards-compatible with C Tor (new keys are stored in the format used by C
  Tor, and any existing keys are expected to be in this format too).

In the future we plan to also support HSM-based key stores.

## Key specifiers and key types

The [`Keystore`] APIs expect a "key specifier" (specified for each supported key
type via the [`KeySpecifier`] trait), and a [`KeyType`].

A "key specifier" identifies a group of equivalent keys, each of a different
type (algorithm). It is used to determine the path of the key within the key
store (minus the extension).

[`KeyType`] represents the type of a key (e.g. "Ed25519 keypair").
[`KeyType::arti_extension`] specifies what file extension keys of that type are
expected to have (when stored in an Arti store).

The [`KeySpecifier::arti_path`] and [`KeyType::arti_extension`] are joined
to form the path of the key on disk (relative to the root dir of the key store).
This enables the key stores to have multiple keys with the same role (i.e. the
same `KeySpecifier::arti_path`), but different key types (i.e. different
`KeyType::arti_extension`s).

`KeySpecifier` implementers must specify:
* `arti_path`: the location of the key in the Arti key store. This also serves
  as a unique identifier for a particular instance of a key.
* `ctor_path`: the location of the key in the C Tor key store (optional).

## Feature flags

### Additive features

(None yet.)

### Experimental and unstable features

 Note that the APIs enabled by these features are NOT covered by semantic
 versioning[^1] guarantees: we might break them or remove them between patch
 versions.

* `keymgr` -- build with full key manager support. Disabling this
  feature causes `tor-keymgr` to export a no-op, placeholder implementation.

[^1]: Remember, semantic versioning is what makes various `cargo`
features work reliably. To be explicit: if you want `cargo update`
to _only_ make safe changes, then you cannot enable these
features.
