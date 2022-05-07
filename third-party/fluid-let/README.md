fluid-let
=========

[![Build Status](https://github.com/ilammy/fluid-let/workflows/Tests/badge.svg)](https://github.com/ilammy/fluid-let/actions)
[![Rust Documentation](https://docs.rs/fluid-let/badge.svg)](https://docs.rs/fluid-let)
[![Latest Version](https://img.shields.io/crates/v/fluid-let.svg)](https://crates.io/crates/fluid-let)

[**fluid-let**](https://crates.io/crates/fluid-let) implements _dynamically scoped_ variables.

Dynamic or _fluid_ variables are a handy way to define global configuration values.
They come from the Lisp family of languages where they are relatively popular for this use case.

## Usage

Add this to your Cargo.toml:

```toml
[dependencies]
fluid-let = "1"
```

You can declare global dynamic variables using `fluid_let!` macro.
Suppose you want to have a configurable `Debug` implementation for your hashes,
controlling whether to print out the whole hash or a truncated version:

```rust
use fluid_let::fluid_let;

fluid_let!(pub static DEBUG_FULL_HASH: bool);
```

Enable full print out using the `fluid_set!` macro.
Assignments to dynamic variables are effective for a certain _dynamic_ scope.
In this case, while the function is being executed:

```rust
use fluid_let::fluid_set;

fn some_important_function() {
    fluid_set!(DEBUG_FULL_HASH, &true);

    // Hashes will be printed out with full precision in this function
    // as well as in all functions that it calls.
}
```

And here is how you can implement `Debug` that uses dynamic configuration:

```rust
impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let full = DEBUG_FULL_HASH.copied().unwrap_or(false);

        write!(f, "Hash(")?;
        if full {
            for byte in &self.value {
                write!(f, "{:02X}", byte)?;
            }
        } else {
            for byte in &self.value[..4] {
                write!(f, "{:02X}", byte)?;
            }
            write!(f, "...")?;
        }
        write!(f, ")")
    }
}
```

Here we print either the full value of the hash, or a truncated version,
based on whether debugging mode has been enabled by the caller or not.

## License

The code is licensed under **MIT license** (see [LICENSE](LICENSE)).
