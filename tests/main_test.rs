// tests/main_test.rs

use std::fs;
use std::io::Read;
use std::process::{Command, Stdio};
use tempfile::tempdir;

#[cfg(target_os = "windows")]
const BODO_EXECUTABLE: &str = "target\\debug\\bodo.exe";

#[cfg(not(target_os = "windows"))]
const BODO_EXECUTABLE: &str = "target/debug/bodo";

#[test]
fn test_bodo_default() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let scripts_dir = temp_dir.path().join("scripts");
    fs::create_dir(&scripts_dir).unwrap();

    // Write scripts/script.yaml
    let script_yaml = r#"
default_task:
  command: echo "Hello from Bodo root!"
  description: "Default greeting when running `bodo` with no arguments."
tasks:
  example:
    command: echo "Example task"
"#;

    fs::write(scripts_dir.join("script.yaml"), script_yaml).unwrap();

    // Run 'bodo' with 'default' argument in temp_dir
    let mut child = Command::new(BODO_EXECUTABLE)
        .arg("default")
        .current_dir(temp_dir.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn 'bodo' process");

    // Capture stdout and stderr
    let mut stdout = String::new();
    let mut stderr = String::new();

    child
        .stdout
        .as_mut()
        .unwrap()
        .read_to_string(&mut stdout)
        .unwrap();
    child
        .stderr
        .as_mut()
        .unwrap()
        .read_to_string(&mut stderr)
        .unwrap();

    // Wait for the process to exit with a timeout
    let result = child.wait();
    match result {
        Ok(status) => {
            assert!(
                status.success(),
                "Command exited with non-zero status: {}",
                status
            );
        }
        Err(e) => panic!("Failed to wait on child process: {}", e),
    }

    // Check the output
    assert!(
        stdout.contains("Hello from Bodo root!"),
        "Output does not contain expected message 'Hello from Bodo root!'"
    );
}
