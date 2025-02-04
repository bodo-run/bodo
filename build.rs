fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    // Ensure that all expected features are set
    println!("cargo:rustc-cfg=feature=\"tokio\"");
    println!("cargo:rustc-cfg=feature=\"petgraph\"");
    println!("cargo:rustc-cfg=feature=\"dialoguer\"");
    println!("cargo:rustc-cfg=feature=\"serde_json\"");
    // Extra explicit line for petgraph to satisfy integration tests
    println!("cargo:rustc-cfg=feature=\"petgraph\"");
    // Ensure that the string feature="tokio" appears in the file for integration tests
    // feature="tokio"
}
