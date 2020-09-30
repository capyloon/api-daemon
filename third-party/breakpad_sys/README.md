# breakpad-sys

## Overview
rust wrapper for google breakpad

## Build
host compile:
- cargo build

cross compile for arm:
- GONK_DIR=[gonk/dir/path] cargo build --target=armv7-linux-androideabi


## Example
initialize exception handler for crash signal and panic handler for rust panic: 

```rust
use breakpad_sys::{init_breakpad, write_minidump};
use std::panic;

let exception_handler = init_breakpad("minidump_path".into());
// Write minidump while panic
panic::set_hook(Box::new(move |_| {
	write_minidump(exception_handler);
}));


