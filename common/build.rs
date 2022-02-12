/// Generate Rust code for the protobuf messages.
use std::env;
use std::path::Path;

fn main() {
    // Add a Linking path for native lib.
    if env::var("BUILD_WITH_NDK_DIR").is_ok() {
        let path = env::var("CARGO_MANIFEST_DIR").unwrap();
        println!(
            "cargo:rustc-link-search=native={}",
            Path::new(&path)
                .join("libnative")
                .join(env::var("TARGET").unwrap())
                .display()
        );
    }
}
