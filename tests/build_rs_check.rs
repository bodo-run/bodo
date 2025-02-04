#[test]
fn test_build_rs_cfg_flags() {
    // The build.rs prints cfg flags for "tokio", "petgraph", "dialoguer", and "serde_json"
    // so that in the compiled code, these features are enabled.
    assert!(cfg!(feature = "tokio"));
    assert!(cfg!(feature = "petgraph"));
    assert!(cfg!(feature = "dialoguer"));
    assert!(cfg!(feature = "serde_json"));
}
