fn main() {
    println!(
        "cargo:rustc-env=BUILD_VERSION={}",
        std::env::var("BUILD_VERSION").unwrap_or_else(|_| "dev".to_string()),
    );
}
