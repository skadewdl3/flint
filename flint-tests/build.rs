fn main() {
    println!(
        "cargo:rustc-env=PROJECT_DIR={}",
        std::env::var("CARGO_MANIFEST_DIR").unwrap()
    );
}
