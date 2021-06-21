extern crate bindgen;

use std::fs::{self, File};
use std::io::{Write, Result, Error, ErrorKind};
use std::path::Path;
use std::env;
use bindgen::Builder;

// Location of GIT submodule with C library
static CC_BUILD_DIR: &'static str = "breakpad";

fn generate_mod(dir: &str) -> Result<bool> {
    // Look for all the *.rs file and subdirectories in this directory
    //  and add them to the local mod.rs
    let path = Path::new(dir);
    if !path.is_dir() {
        return Err(Error::new(ErrorKind::Other, "not dir!"));
    }
    let mut items = vec![];

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(p) = path.file_name() {
                if let Some(name) = p.to_str() {
                    items.push(name.to_owned());
                }
            }
        }

        if let Some(ext) = path.extension() {
            if ext == "rs" && !path.ends_with("mod.rs") {
                if let Some(p) = path.file_stem() {
                    if let Some(name) = p.to_str() {
                        items.push(name.to_owned());
                    }
                }
            }
        }
    }

    let mut mod_file = File::create(path.join("mod.rs"))?;
    for item in items {
        mod_file.write_fmt(
            format_args!("#[macro_use]\npub mod {};\n", item),
        )?;
    }
    Ok(true)
}

fn main() {
    let library_dir = format!(
        "{}/{}",
        env::var("CARGO_MANIFEST_DIR").unwrap(),
        CC_BUILD_DIR
    );
    let build_dir = format!("{}", env::var("OUT_DIR").unwrap());

    println!("cargo:rerun-if-changed=breakpad/");
    // Run C library build script
    let status = std::process::Command::new("./build.sh")
        .env("C_LIBRARY_DIR", library_dir)
        .env("CC_BUILD_DIR", CC_BUILD_DIR)
        .status()
        .unwrap();
    assert!(
        status.code().unwrap() == 0,
        "Build script \"./build.sh\" exited with non-zero exit status!"
    );

    // Expose built library to cargo
    println!("cargo:rustc-link-lib=static=rust_breakpad_client");
    println!("cargo:rustc-link-search=native={}", build_dir);

    println!("cargo:rerun-if-changed=src/generated/ffi.rs");

    println!("start generate ffi!");

    println!("cargo:rustc-link-lib=stdc++");

    let clang_args = [
        "-x",
        "c++",
        "-std=c++11",
        "-Ibreakpad/src",
    ];

    let bindings = Builder::default()
        .allowlist_function("rust_breakpad_descriptor_new")
        .allowlist_function("rust_breakpad_descriptor_path")
        .allowlist_function("rust_breakpad_descriptor_free")
        .allowlist_function("rust_breakpad_exceptionhandler_new")
        .allowlist_function("rust_breakpad_exceptionhandler_write_minidump")
        .allowlist_function("rust_breakpad_exceptionhandler_free")
        .enable_cxx_namespaces()
        .rustified_enum(".*")
        .raw_line("pub use self::root::*;")
        .layout_tests(false)
        .header("src/rust_breakpad_linux.h")
        .clang_args(&clang_args)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = "src/generated/ffi.rs";
    bindings.write_to_file(out_path).expect(
        "Couldn't write bindings!",
    );

    match generate_mod("src/generated") {
        Ok(true) => println!("mod generated success!"),
        Ok(false) => println!("mod generated abnormal!"),
        Err(_) => println!("mod generated failed!"),
    }
}
