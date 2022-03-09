# tor-units

`tor-units` -- Safe wrappers for primitive numeric types.

## Overview

This crate is part of
[Arti](https://gitlab.torproject.org/tpo/core/arti/), a project to
implement [Tor](https://www.torproject.org/) in Rust.
It provides safe wrappers for primitive numeric wrappers used in
other parts of Arti.
In particular, it provides:
  * a bounded i32 with both checked and clamping constructors,
  * an integer milliseconds wrapper with conversion to [`Duration`]
  * an integer seconds wrapper with conversion to [`Duration`]
  * a percentage wrapper, to prevent accidental failure
    to divide by 100.
  * a SendMeVersion which can be compared only.

License: MIT OR Apache-2.0
