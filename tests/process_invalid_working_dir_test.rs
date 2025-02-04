use bodo::process::ProcessManager;

#[test]
fn test_spawn_command_invalid_working_dir() {
    let mut pm = ProcessManager::new(false);
    // Provide a non-existent directory; on many systems this will cause an error when spawning.
    let result = pm.spawn_command(
        "test_invalid",
        "echo test",
        false,
        None,
        None,
        Some("nonexistent_dir"),
    );
    assert!(
        result.is_err(),
        "Expected error for invalid working directory"
    );
}
