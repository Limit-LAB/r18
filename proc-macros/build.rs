fn main() {
    if std::env::var("RUSTUP_TOOLCHAIN")
        .map(|t| t.starts_with("nightly"))
        .unwrap_or(false)
    {
        println!("cargo:rustc-cfg=feature=\"nightly-features\"");
    }
}
