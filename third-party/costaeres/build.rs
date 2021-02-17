fn main() {
    println!("cargo:rerun-if-changed=./db/migrations/00001_main.sql");
}
