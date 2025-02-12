// tests/process_test.rs

use bodo::process::ProcessManager;

#[test]
fn test_process_manager_spawn_and_run() {
    let mut pm = ProcessManager::new(false);
    pm.spawn_command("test_echo", "echo Hello", false, None, None, None)
        .unwrap();
    pm.run_concurrently().unwrap();
}

#[test]
fn test_process_manager_fail_fast() {
    let mut pm = ProcessManager::new(true);
    pm.spawn_command("fail_cmd", "false", false, None, None, None)
        .unwrap();
    pm.spawn_command("echo_cmd", "echo Should not run", false, None, None, None)
        .unwrap();

    let result = pm.run_concurrently();
    assert!(result.is_err());
}

#[test]
fn test_process_manager_kill_all() {
    let mut pm = ProcessManager::new(false);
    pm.spawn_command("sleep_cmd", "sleep 5", false, None, None, None)
        .unwrap();
    pm.kill_all().unwrap();
}

#[test]
fn test_process_manager_no_fail_fast() {
    let mut pm = ProcessManager::new(false);
    pm.spawn_command("fail_cmd", "false", false, None, None, None)
        .unwrap();
    pm.spawn_command(
        "echo_cmd",
        "echo Should run even if previous fails",
        false,
        None,
        None,
        None,
    )
    .unwrap();

    let result = pm.run_concurrently();
    assert!(result.is_err(), "Expected an error due to failed process");
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
        None,
    );
    assert!(result.is_ok());
    pm.run_concurrently().unwrap();
    // Should not panic or crash due to invalid color
}
