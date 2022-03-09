use std::env;
use std::fs;
use std::path::PathBuf;

const SKIP_FILENAMES: &[&str] = &["crc32_small", "crc64_small"];

fn main() {
    let target = env::var("TARGET").unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=LZMA_API_STATIC");
    let want_static = cfg!(feature = "static") || env::var("LZMA_API_STATIC").is_ok();
    let msvc = target.contains("msvc");

    // If a static link is desired, we compile from source.
    // If we're compiling for MSVC, pkg-config runs a risk of picking up MinGW
    // libraries by accident, so disable it.
    //
    // Otherwise check the system to see if it has an lzma library already
    // installed that we can use.
    if !want_static && !msvc && pkg_config::probe_library("liblzma").is_ok() {
        return;
    }

    let out_dir = env::var("OUT_DIR").unwrap();
    println!("cargo:root={}", out_dir);
    let include_dir = env::current_dir().unwrap().join("xz-5.2/src/liblzma/api");
    println!("cargo:include={}", include_dir.display());

    let src_files = [
        "xz-5.2/src/liblzma/common",
        "xz-5.2/src/liblzma/lzma",
        "xz-5.2/src/liblzma/lz",
        "xz-5.2/src/liblzma/check",
        "xz-5.2/src/liblzma/delta",
        "xz-5.2/src/liblzma/rangecoder",
        "xz-5.2/src/liblzma/simple",
    ]
    .iter()
    .flat_map(|dir| read_dir_files(dir))
    .chain(vec![
        "xz-5.2/src/common/tuklib_cpucores.c".into(),
        "xz-5.2/src/common/tuklib_physmem.c".into(),
    ]);

    let mut build = cc::Build::new();

    build
        .files(src_files)
        // all C preproc defines are in `./config.h`
        .define("HAVE_CONFIG_H", "1")
        .include("xz-5.2/src/liblzma/api")
        .include("xz-5.2/src/liblzma/lzma")
        .include("xz-5.2/src/liblzma/lz")
        .include("xz-5.2/src/liblzma/check")
        .include("xz-5.2/src/liblzma/simple")
        .include("xz-5.2/src/liblzma/delta")
        .include("xz-5.2/src/liblzma/common")
        .include("xz-5.2/src/liblzma/rangecoder")
        .include("xz-5.2/src/common")
        .include(env::current_dir().unwrap());

    if !target.ends_with("msvc") {
        build.flag("-std=c99").flag("-pthread");
    }

    if let Ok(s) = env::var("CARGO_CFG_TARGET_ENDIAN") {
        if s == "big" {
            build.define("WORDS_BIGENDIAN", None);
        }
    }

    build.compile("liblzma.a");
}

fn read_dir_files(dir: &str) -> impl Iterator<Item = PathBuf> {
    fs::read_dir(dir)
        .expect(&format!("failed to read dir {}", dir))
        .filter_map(|ent| {
            let ent = ent.expect("failed to read entry");

            if ent.file_type().unwrap().is_dir() {
                return None;
            }

            let path = ent.path();

            if path.extension().unwrap() != "c" {
                return None;
            }

            {
                let file_stem = path.file_stem().unwrap().to_str().unwrap();
                if SKIP_FILENAMES.contains(&file_stem) {
                    return None;
                }
                if file_stem.ends_with("tablegen") {
                    return None;
                }
            }

            Some(path)
        })
}
