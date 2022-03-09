# tor-linkspec

`tor-linkspec`: Descriptions of Tor relays, as used to connect to them.

## Overview

The `tor-linkspec` crate provides traits and data structures that
describe how to connect to Tor relays.

When describing the location of a Tor relay on the network, the
Tor protocol uses a set of "link specifiers", each of which
corresponds to a single aspect of the relay's location or
identity—such as its IP address and port, its Ed25519 identity
key, its (legacy) RSA identity fingerprint, or so on.  This
crate's [`LinkSpec`] type encodes these structures.

When a client is building a circuit through the Tor network, it
needs to know certain information about the relays in that
circuit.  This crate's [`ChanTarget`] and [`CircTarget`] traits
represent objects that describe a relay on the network that a
client can use as the first hop, or as any hop, in a circuit.

This crate is part of
[Arti](https://gitlab.torproject.org/tpo/core/arti/), a project to
implement [Tor](https://www.torproject.org/) in Rust.  Several
other crates in Arti depend on it.  You will probably not need
this crate yourself unless you are interacting with the Tor
protocol at a fairly low level.

`tor-linkspec` is a separate crate so that it can be used by other
crates that expose link specifiers and by crates that consume
them.

### Future work

TODO: Possibly we should rename this crate.  "Linkspec" is a
pretty esoteric term in the Tor protocols.

TODO: Possibly the link specifiers and the `*Target` traits belong in different crates.

License: MIT OR Apache-2.0
