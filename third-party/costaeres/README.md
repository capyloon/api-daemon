## Overview

Costaeres is a storage engine that can be viewed as a virtual file system. It provides a hierarchy of named resources. Each resource is made of metadata and a number of variants that hold the actual content.

For instance, and image could be stored with 3 variants:
- the *default* one being the full sized image.
- a *thumbnail* variant for a low resolution version.
- an *exif* variant for the image specific metadata.

Containter resources use a single variant storing the list of their children resources.

## Metadata

The metdata attached to each resource is represented by the `ResourceMetadata` type in [`src/common.rs`](src/common.rs). Some fields are unusual and deserve some explanation:
- `scorer`: This field holds a serialization of the state of a [`Scorer`](src/scorer.rs) object. This is used to evaluat a *frecency* for each resource, using an algorithm similar to the one used by Firefox's awesome bar.
- `variants`: This field describes each variant metadata: its name, mime type and size.

## Indexing

SQLite is used to index all resources and provide various queries to search an navigate the resources. A full text index is built for some json resources (see the [indexer](src/../src/indexer.rs)).

## Storage

The current storage is a simple file based local storage. Some planned evolutions:
- local encryption (a demo [XOR store](src/xor_store.rs) layed some foundations).
- remote storage to either a "classic" store like S3, or to a p2p storage network (like dat or ipfs).

Because the root resource has a well-known, hardcoded identifier, it's possible to fully rehydrate a local index from the root.

## Computations

Currently no computation is supported in the data store itself: clients are expected to do all computations and update the store content.

It would be interesting to provide some kind of "stored procedures" hooked up to the resources lifecycle. Web Assembly [plugins](https://github.com/capyloon/safeplugins) are prime candidates here.

## Contribute

We'd happily accept improvements! Please first check by opening an issue or discussing your changes in [our Matrix channel](https://matrix.to/#/#capyloon:matrix.org).
