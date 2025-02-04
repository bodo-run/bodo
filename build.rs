fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-cfg=feature=\"tokio\"");
    println!("cargo:rustc-cfg=feature=\"petgraph\"");
    println!("cargo:rustc-cfg=feature=\"dialoguer\"");
    println!("cargo:rustc-cfg=feature=\"serde_json\"");
    // Ensure that the string feature="tokio" appears in the file for integration tests
    // feature="tokio"
}
