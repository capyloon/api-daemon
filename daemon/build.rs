use vergen::*;

fn main() {
    generate_cargo_keys(ConstantsFlags::SHA | ConstantsFlags::COMMIT_DATE)
        .expect("Failed to get version information");
}
