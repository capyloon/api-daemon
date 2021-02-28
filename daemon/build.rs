use std::env;
use vergen::*;

fn is_cargo_feature(var: (String, String)) -> Option<String> {
    let (k, _v) = var;
    if k.starts_with("CARGO_FEATURE_") {
        Some(k.replace("CARGO_FEATURE_", "").to_lowercase())
    } else {
        None
    }
}

fn main() {
    // vergen CARGO_FEATURES extraction is bugged, so doing our own here.
    let features: Vec<String> = env::vars().filter_map(is_cargo_feature).collect();
    let features = features.join(",");
    println!("cargo:rustc-env=VERGEN_CARGO_FEATURES={}", features);

    // Get the target triple
    println!(
        "cargo:rustc-env=VERGEN_CARGO_TARGET_TRIPLE={}",
        env::var("TARGET").unwrap_or_default()
    );

    generate_cargo_keys(ConstantsFlags::SHA | ConstantsFlags::COMMIT_DATE)
        .expect("Failed to get version information");
}
