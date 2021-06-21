# mkstemp.rs

Safe wrapper over mkstemp function from libc

[Documentation](https://dremon.github.io/mkstemp.rs/doc/mkstemp)

Usage example:

```rust
use std::io::Write;
extern crate mkstemp;
pub fn main() {
    // delete automatically when it goes out of scope
    let mut temp_file = mkstemp::TempFile::new("/tmp/testXXXXXX", true).unwrap();
     temp_file.write("test content".as_bytes()).unwrap();
}
```

## License

Licensed under MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
