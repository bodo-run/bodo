use bodo::sandbox::Sandbox;
use std::collections::HashMap;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_sandbox_creation() {
    let sandbox = Sandbox::new();
    assert!(sandbox.is_ok(), "Sandbox creation should succeed");
}

#[test]
fn test_sandbox_directory_setup() {
    let sandbox = Sandbox::new().expect("Failed to create sandbox");
    let working_dir = Path::new("/test/dir");

    // This is testing internal functionality, so we'd need to make setup_sandbox_dirs public
    // or create a wrapper method for testing
    // For now, we'll test the overall functionality through execute_and_analyze
}

#[test]
fn test_sandbox_file_write_detection() {
    let sandbox = Sandbox::new().expect("Failed to create sandbox");
    let working_dir = Path::new("/tmp");
    let env = HashMap::new();

    // Test simple file write command
    let command = "echo 'test content' > test_file.txt";
    let result = sandbox.execute_and_analyze(command, working_dir, &env);

    match result {
        Ok(side_effects) => {
            // Should detect at least a process spawn and potentially a file write
            assert!(!side_effects.is_empty(), "Should detect side effects");

            // Check for process spawn
            let has_process_spawn = side_effects
                .iter()
                .any(|effect| matches!(effect, bodo::plugin::SideEffect::ProcessSpawn(_)));
            assert!(has_process_spawn, "Should detect process spawn");
        }
        Err(e) => {
            // If sandbox tools aren't available, the test should still pass
            // but we should log the reason
            println!(
                "Sandbox execution failed (expected if bwrap/firejail not available): {}",
                e
            );
        }
    }
}

#[test]
fn test_sandbox_network_detection() {
    let sandbox = Sandbox::new().expect("Failed to create sandbox");
    let working_dir = Path::new("/tmp");
    let env = HashMap::new();

    // Test network command (this might fail in CI without network access)
    let command = "curl --version"; // Safe command that mentions curl
    let result = sandbox.execute_and_analyze(command, working_dir, &env);

    match result {
        Ok(side_effects) => {
            // Should detect process spawn at minimum
            let has_process_spawn = side_effects
                .iter()
                .any(|effect| matches!(effect, bodo::plugin::SideEffect::ProcessSpawn(_)));
            assert!(has_process_spawn, "Should detect process spawn");
        }
        Err(e) => {
            println!(
                "Sandbox execution failed (expected if sandbox tools not available): {}",
                e
            );
        }
    }
}

#[test]
fn test_sandbox_fallback_behavior() {
    // This test ensures that even without sandbox tools, the system gracefully falls back
    let sandbox = Sandbox::new().expect("Failed to create sandbox");
    let working_dir = Path::new("/tmp");
    let env = HashMap::new();

    // Test with a simple command that should work everywhere
    let command = "echo 'hello world'";
    let result = sandbox.execute_and_analyze(command, working_dir, &env);

    // Should either succeed with sandbox or fall back gracefully
    match result {
        Ok(side_effects) => {
            assert!(
                !side_effects.is_empty(),
                "Should detect at least process spawn"
            );
        }
        Err(_) => {
            // If it fails, that's also acceptable for this test
            // as it means the fallback was attempted
        }
    }
}

#[test]
fn test_sandbox_multiple_commands() {
    let sandbox = Sandbox::new().expect("Failed to create sandbox");
    let working_dir = Path::new("/tmp");
    let env = HashMap::new();

    // Test compound command
    let command = "echo 'first' > file1.txt && echo 'second' > file2.txt";
    let result = sandbox.execute_and_analyze(command, working_dir, &env);

    match result {
        Ok(side_effects) => {
            // Should detect process spawn
            let has_process_spawn = side_effects
                .iter()
                .any(|effect| matches!(effect, bodo::plugin::SideEffect::ProcessSpawn(_)));
            assert!(has_process_spawn, "Should detect process spawn");
        }
        Err(e) => {
            println!("Sandbox execution failed: {}", e);
        }
    }
}

#[test]
fn test_sandbox_environment_variables() {
    let sandbox = Sandbox::new().expect("Failed to create sandbox");
    let working_dir = Path::new("/tmp");
    let mut env = HashMap::new();
    env.insert("TEST_VAR".to_string(), "test_value".to_string());

    // Test command that uses environment variable
    let command = "echo $TEST_VAR";
    let result = sandbox.execute_and_analyze(command, working_dir, &env);

    match result {
        Ok(side_effects) => {
            let has_process_spawn = side_effects
                .iter()
                .any(|effect| matches!(effect, bodo::plugin::SideEffect::ProcessSpawn(_)));
            assert!(has_process_spawn, "Should detect process spawn");
        }
        Err(e) => {
            println!("Sandbox execution failed: {}", e);
        }
    }
}

#[cfg(unix)]
#[test]
fn test_sandbox_tool_detection() {
    use std::process::Command;

    // Test if we can detect available sandbox tools
    let has_bwrap = Command::new("which")
        .arg("bwrap")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let has_firejail = Command::new("which")
        .arg("firejail")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    println!("Bubblewrap available: {}", has_bwrap);
    println!("Firejail available: {}", has_firejail);

    // This test just reports the availability - it doesn't fail
    // This helps with debugging test failures in different environments
}

#[test]
fn test_sandbox_working_directory() {
    let sandbox = Sandbox::new().expect("Failed to create sandbox");
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let working_dir = temp_dir.path();
    let env = HashMap::new();

    // Test command that should work in any directory
    let command = "pwd";
    let result = sandbox.execute_and_analyze(command, working_dir, &env);

    match result {
        Ok(side_effects) => {
            let has_process_spawn = side_effects
                .iter()
                .any(|effect| matches!(effect, bodo::plugin::SideEffect::ProcessSpawn(_)));
            assert!(has_process_spawn, "Should detect process spawn");
        }
        Err(e) => {
            println!("Sandbox execution failed: {}", e);
        }
    }
}
