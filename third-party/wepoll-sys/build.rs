use std::env;
use std::io;
use std::path::Path;

fn main() {
    let src_dir = Path::new("wepoll");
    let out_env_var =
        env::var("OUT_DIR").expect("Failed to obtain the OUT_DIR variable");

    let out_dir = Path::new(&out_env_var);
    let build_dir = out_dir.join("wepoll-build");

    if let Err(err) = std::fs::remove_dir_all(&build_dir) {
        if err.kind() != io::ErrorKind::NotFound {
            panic!("Failed to remove the build directory: {}", err);
        }
    }

    std::fs::create_dir(&build_dir)
        .expect("Failed to create the build directory");

    for file in &["wepoll.c", "wepoll.h"] {
        std::fs::copy(src_dir.join(file), build_dir.join(file))
            .expect(&format!("Failed to copy {} to the build directory", file));
    }

    cc::Build::new()
        .include(&build_dir)
        .out_dir(&build_dir)
        .file(&build_dir.join("wepoll.c"))
        .compile("wepoll");

    println!("cargo:rustc-link-lib=static=wepoll");
    println!("cargo:rustc-link-search={}", &build_dir.display());
}
