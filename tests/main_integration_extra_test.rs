use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_bodo_integration_default() {
    if std::env::var("CARGO_BIN_EXE_bodo").is_err() {
        eprintln!("Skipping integration test because Bodo binary not found");
        return;
    }
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let script_content = r#"
default_task:
  command: echo "Integration default task"
  description: "Default task for integration"
"#;
    let script_path = temp_dir.path().join("script.yaml");
    std::fs::write(&script_path, script_content).expect("Failed to write script.yaml");

    let exe = std::env::var("CARGO_BIN_EXE_bodo").expect("Bodo binary not found");
    let output = Command::new(exe)
        .env("BODO_ROOT_SCRIPT", script_path.to_str().unwrap())
        .env("BODO_NO_WATCH", "1")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to run Bodo");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Integration default task"),
        "Integration output mismatch"
    );
}

#[test]
fn test_bodo_integration_list() {
    if std::env::var("CARGO_BIN_EXE_bodo").is_err() {
        eprintln!("Skipping integration list test because Bodo binary not found");
        return;
    }
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let script_content = r#"
default_task:
  command: echo "Default"
  description: "Default task"

tasks:
  sample:
    command: echo "Sample task"
    description: "A sample task for listing"
"#;
    let script_path = temp_dir.path().join("script.yaml");
    std::fs::write(&script_path, script_content).expect("Failed to write script.yaml");

    let exe = std::env::var("CARGO_BIN_EXE_bodo").expect("Bodo binary not found");
    let output = Command::new(exe)
        .arg("--list")
        .env("BODO_ROOT_SCRIPT", script_path.to_str().unwrap())
        .env("BODO_NO_WATCH", "1")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to run Bodo with --list");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Default task"),
        "List output missing default task"
    );
    assert!(
        stdout.contains("A sample task for listing"),
        "List output missing sample task"
    );
}
