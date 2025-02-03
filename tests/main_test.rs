// tests/main_test.rs

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

#[test]
fn test_bodo_default() {
    // First, ensure 'bodo' binary is built
    let status = Command::new("cargo")
        .args(["build", "--bin", "bodo"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("Failed to execute cargo build command");
    assert!(status.success(), "Cargo build failed");

    // Build the path to the built 'bodo' executable
    let exe_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("debug")
        .join("bodo");
    #[cfg(windows)]
    exe_path.set_extension("exe");

    let mut child = Command::new(exe_path)
        .arg("default")
        .env("RUST_LOG", "info")
        .env("BODO_NO_WATCH", "1")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
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
                assert!(status.success());
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!("STDOUT:\n{}", stdout);
                println!("STDERR:\n{}", stderr);
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
    // First, ensure 'bodo' binary is built
    let status = Command::new("cargo")
        .args(["build", "--bin", "bodo"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("Failed to execute cargo build command");
    assert!(status.success(), "Cargo build failed");

    // Build the path to the built 'bodo' executable
    let mut exe_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    exe_path.push("target");
    exe_path.push("debug");
    exe_path.push("bodo");
    #[cfg(windows)]
    exe_path.set_extension("exe");

    let mut child = Command::new(exe_path)
        .arg("--list")
        .env("RUST_LOG", "info")
        .env("BODO_NO_WATCH", "1")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
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
                assert!(status.success());
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!("STDOUT:\n{}", stdout);
                println!("STDERR:\n{}", stderr);
                let output_combined = format!("{}{}", stdout, stderr);
                assert!(
                    output_combined
                        .contains("Default greeting when running `bodo` with no arguments."),
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
