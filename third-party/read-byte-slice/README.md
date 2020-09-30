[![Build Status](https://travis-ci.org/luser/read-byte-slice.svg?branch=master)](https://travis-ci.org/luser/read-byte-slice) [![crates.io](https://img.shields.io/crates/v/read-byte-slice.svg)](https://crates.io/crates/read-byte-slice) [![](https://docs.rs/read-byte-slice/badge.svg)](https://docs.rs/read-byte-slice)

This crate implements a type `ByteSliceIter` that reads bytes from a reader and allows iterating
over them as slices with a maximum length, similar to the [`chunks`] method on slices.

It is implemented as a [`FallibleStreamingIterator`] so that it can reuse its buffer and not
allocate for each chunk. (That trait is re-exported here for convenience.)

# Example
```rust
extern crate read_byte_slice;

use read_byte_slice::{ByteSliceIter, FallibleStreamingIterator};
use std::io;

fn work() -> io::Result<()> {
  let bytes = b"0123456789abcdef0123456789abcdef";
  // Iterate over the bytes in 8-byte chunks.
  let mut iter = ByteSliceIter::new(&bytes[..], 8);
  while let Some(chunk) = iter.next()? {
    println!("{:?}", chunk);
  }
  Ok(())
}

fn main() {
  work().unwrap();
}
```

# License

`read-byte-slice` is distributed under the terms of both the MIT license and
the Apache License (Version 2.0).

See LICENSE-APACHE and LICENSE-MIT for details.

[`chunks`]: https://doc.rust-lang.org/std/primitive.slice.html#method.chunks
[`FallibleStreamingIterator`]: https://docs.rs/fallible-streaming-iterator/*/fallible_streaming_iterator/trait.FallibleStreamingIterator.html
