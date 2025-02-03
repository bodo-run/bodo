use bodo::process::ProcessManager;

#[test]
fn test_spawn_command_success() {
    let mut pm = ProcessManager::new(true);
    let result = pm.spawn_command("echo_test", "echo 'Hello'", false, None, None);
    assert!(result.is_ok());
    pm.run_concurrently().unwrap();
}

#[test]
fn test_spawn_command_failure() {
    let mut pm = ProcessManager::new(true);
    let result = pm.spawn_command("fail_test", "exit 1", false, None, None);
    assert!(result.is_ok());
    let run_result = pm.run_concurrently();
    assert!(run_result.is_err());
}

#[test]
fn test_kill_all_processes() {
    let mut pm = ProcessManager::new(true);
    pm.spawn_command("sleep_test", "sleep 5", false, None, None)
        .unwrap();
    pm.kill_all().unwrap();
}

#[test]
fn test_spawn_command_with_invalid_prefix_color() {
    let mut pm = ProcessManager::new(true);
    // Using an invalid color name
    let result = pm.spawn_command(
        "color_test",
        "echo 'Testing invalid color'",
        true,
        Some("label".to_string()),
        Some("invalid_color".to_string()),
    );
    assert!(result.is_ok());
    pm.run_concurrently().unwrap();
    // Should not panic or crash due to invalid color
}
