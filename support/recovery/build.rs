use std::env;
use std::path::Path;

fn main() {
    if let Ok(_) = env::var("BUILD_WITH_NDK_DIR") {
        let path = env::var("CARGO_MANIFEST_DIR").unwrap();
        println!(
            "cargo:rustc-link-search=native={}",
            Path::new(&path).join("libnative").join(env::var("TARGET").unwrap()).display()
        );
    }
}
