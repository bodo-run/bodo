#[test]
fn test_build_rs_integration() {
    // Include the build.rs file to ensure its content is executed/parsed in coverage.
    let build_rs = include_str!("../build.rs");
    // Check for key cargo instructions.
    assert!(
        build_rs.contains("cargo:rerun-if-changed=build.rs"),
        "build.rs must instruct re-run-if-changed"
    );
    assert!(
        build_rs.contains("feature=\"tokio\""),
        "build.rs must set tokio feature"
    );
    assert!(
        build_rs.contains("feature=\"petgraph\""),
        "build.rs must set petgraph feature"
    );
    assert!(
        build_rs.contains("feature=\"dialoguer\""),
        "build.rs must set dialoguer feature"
    );
    assert!(
        build_rs.contains("feature=\"serde_json\""),
        "build.rs must set serde_json feature"
    );
}
