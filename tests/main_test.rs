// tests/main_test.rs

use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn test_bodo_default() {
    // Build the path to the built 'bodo' executable
    let current_exe = std::env::current_exe().expect("Failed to get current exe path");

    // Assuming the path to the built 'bodo' binary is `current_exe/../../../debug/bodo`
    let target_dir = current_exe
        .parent() // deps/
        .and_then(|p| p.parent()) // debug/
        .and_then(|p| p.parent()) // target/
        .expect("Failed to get target directory");

    let exe_path = target_dir.join("debug").join("bodo");
    #[cfg(windows)]
    let exe_path = exe_path.with_extension("exe");

    assert!(
        exe_path.exists(),
        "bodo executable not found at {:?}",
        exe_path
    );

    // Create a temp directory
    let temp_dir = tempdir().expect("Failed to create temp dir");

    // Write the scripts/script.yaml file
    let scripts_dir = temp_dir.path().join("scripts");
    std::fs::create_dir_all(&scripts_dir).expect("Failed to create scripts directory");

    let script_content = r#"
default_task:
  command: echo "Hello from Bodo root!"
  description: "Default greeting when running `bodo` with no arguments."
"#;

    let script_path = scripts_dir.join("script.yaml");
    std::fs::write(&script_path, script_content).expect("Failed to write script.yaml");

    // Set environment variables to point to our temp scripts directory
    let root_script_env = scripts_dir
        .join("script.yaml")
        .to_string_lossy()
        .into_owned();
    let scripts_dirs_env = scripts_dir.to_string_lossy().into_owned();

    let mut child = Command::new(exe_path)
        // .arg("default") // No need to specify 'default' since we're testing the default task
        .env("RUST_LOG", "info")
        .env("BODO_NO_WATCH", "1")
        .env("BODO_ROOT_SCRIPT", &root_script_env)
        .env("BODO_SCRIPTS_DIRS", &scripts_dirs_env)
        .current_dir(temp_dir.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute command");

    // Wait for at most 10 seconds
    let timeout = Duration::from_secs(10);
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        match child.try_wait() {
            Ok(Some(status)) => {
                // Process has exited
                let output = child.wait_with_output().expect("Failed to wait on child");
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!("STDOUT:\n{}", stdout);
                println!("STDERR:\n{}", stderr);
                assert!(status.success(), "Process exited with status {}", status);
                let output_combined = format!("{}{}", stdout, stderr);
                assert!(
                    output_combined.contains("Hello from Bodo root!"),
                    "Output does not contain expected message 'Hello from Bodo root!'"
                );
                return;
            }
            Ok(None) => {
                // Process still running
                thread::sleep(Duration::from_millis(100));
                continue;
            }
            Err(e) => panic!("Error attempting to wait: {}", e),
        }
    }

    // If we get here, the process timed out
    child.kill().expect("Failed to kill process");
    panic!("Process timed out after {} seconds", timeout.as_secs());
}

#[test]
fn test_bodo_list() {
    // Build the path to the built 'bodo' executable
    let current_exe = std::env::current_exe().expect("Failed to get current exe path");

    // Assuming the path to the built 'bodo' binary is `current_exe/../../../debug/bodo`
    let target_dir = current_exe
        .parent() // deps/
        .and_then(|p| p.parent()) // debug/
        .and_then(|p| p.parent()) // target/
        .expect("Failed to get target directory");

    let exe_path = target_dir.join("debug").join("bodo");
    #[cfg(windows)]
    let exe_path = exe_path.with_extension("exe");

    assert!(
        exe_path.exists(),
        "bodo executable not found at {:?}",
        exe_path
    );

    // Create a temp directory
    let temp_dir = tempdir().expect("Failed to create temp dir");

    // Write the scripts/script.yaml file
    let scripts_dir = temp_dir.path().join("scripts");
    std::fs::create_dir_all(&scripts_dir).expect("Failed to create scripts directory");

    let script_content = r#"
default_task:
  command: echo "Hello from Bodo root!"
  description: "Default greeting when running `bodo` with no arguments."

tasks:
  test:
    command: echo "Running tests"
    description: "Run all tests"
  build:
    command: echo "Building project"
    description: "Build the project"
"#;

    let script_path = scripts_dir.join("script.yaml");
    std::fs::write(&script_path, script_content).expect("Failed to write script.yaml");

    // Set environment variables to point to our temp scripts directory
    let root_script_env = scripts_dir
        .join("script.yaml")
        .to_str()
        .unwrap()
        .to_string();
    let scripts_dirs_env = scripts_dir.to_str().unwrap().to_string();

    let mut child = Command::new(exe_path)
        .arg("--list")
        .env("RUST_LOG", "info")
        .env("BODO_NO_WATCH", "1")
        .env("BODO_ROOT_SCRIPT", &root_script_env)
        .env("BODO_SCRIPTS_DIRS", &scripts_dirs_env)
        .current_dir(temp_dir.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute command");

    // Wait for at most 10 seconds
    let timeout = Duration::from_secs(10);
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        match child.try_wait() {
            Ok(Some(status)) => {
                // Process has exited
                let output = child.wait_with_output().expect("Failed to wait on child");
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!("STDOUT:\n{}", stdout);
                println!("STDERR:\n{}", stderr);
                assert!(status.success(), "Process exited with status {}", status);
                let output_combined = format!("{}{}", stdout, stderr);
                assert!(
                    output_combined
                        .contains("Default greeting when running `bodo` with no arguments."),
                    "Output does not contain expected task descriptions"
                );
                assert!(
                    output_combined.contains("Run all tests"),
                    "Output does not contain expected task descriptions"
                );
                assert!(
                    output_combined.contains("Build the project"),
                    "Output does not contain expected task descriptions"
                );
                return;
            }
            Ok(None) => {
                // Process still running
                thread::sleep(Duration::from_millis(100));
                continue;
            }
            Err(e) => panic!("Error attempting to wait: {}", e),
        }
    }

    // If we get here, the process timed out
    child.kill().expect("Failed to kill process");
    panic!("Process timed out after {} seconds", timeout.as_secs());
}
