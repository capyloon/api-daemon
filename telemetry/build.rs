fn main() {
    // Force to rebuild for environment variable change.
    println!("cargo:rerun-if-env-changed=METRICS_KEY");
}
