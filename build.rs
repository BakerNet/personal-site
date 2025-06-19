fn main() {
    // Capture the current timestamp as the build time
    let build_time = chrono::Utc::now().to_rfc3339();

    // Also set as environment variable for use in env! macro
    println!("cargo:rustc-env=BUILD_TIME={}", build_time);

    // Rerun if build.rs changes
    println!("cargo:rerun-if-changed=build.rs");
}

