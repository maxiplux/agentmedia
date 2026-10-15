fn main() {
    println!(
        "cargo::rustc-env=FRAMECHORD_TARGET={}",
        std::env::var("TARGET").unwrap()
    );
    println!("cargo::rerun-if-changed=build.rs");
    // Use the target, not the build host, so cross-compilation behaves the same.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
    {
        // Debug CLI dispatch + typed YAML loading exceeds MSVC's 1 MiB default.
        // Reserve 8 MiB for the executable's main thread; leave initial commit
        // unchanged. This also applies to cargo install and packaged binaries,
        // without requiring users to set environment variables.
        println!("cargo::rustc-link-arg-bin=framechord=/STACK:8388608");
    }
}
