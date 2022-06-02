# TextNonce

[![Build Status](https://travis-ci.org/mikedilger/textnonce.svg?branch=master)](https://travis-ci.org/mikedilger/textnonce)

Documentation is available at https://docs.rs/textnonce

A nonce is a cryptographic concept of an arbitrary number that is never used
more than once.

`TextNonce` is a nonce because the first 16 characters represents the current
time, which will never have been generated before, nor will it be generated
again, across the period of time in which Timespec is valid.

`TextNonce` additionally includes bytes of randomness, making it difficult
to predict. This makes it suitable to be used for a session ID.

It is also text-based, using only characters in the base64 character set.

Various length `TextNonce`es may be generated.  The minimum length is 16
characters, and lengths must be evenly divisible by 4.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
