use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_main_default_output() {
    if std::env::var("CARGO_BIN_EXE_bodo").is_err() {
        eprintln!("Skipping test_main_default_output because CARGO_BIN_EXE_bodo is not set");
        return;
    }
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let script_content = r#"
default_task:
  command: echo "Integration test default task"
  description: "Integration default"
  
tasks:
  test:
    command: echo "Integration test task"
"#;
    let script_path = temp_dir.path().join("script.yaml");
    std::fs::write(&script_path, script_content).expect("Failed to write script.yaml");

    let exe = std::env::var("CARGO_BIN_EXE_bodo").expect("CARGO_BIN_EXE_bodo not set");
    let output = Command::new(exe)
        .env("BODO_ROOT_SCRIPT", script_path.to_str().unwrap())
        .env("BODO_NO_WATCH", "1")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute main binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Integration test default task"),
        "Output did not contain expected default task output"
    );
}

#[test]
fn test_main_list_option() {
    if std::env::var("CARGO_BIN_EXE_bodo").is_err() {
        eprintln!("Skipping test_main_list_option because CARGO_BIN_EXE_bodo is not set");
        return;
    }
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let script_content = r#"
default_task:
  command: echo "Default"
  description: Default task
  
tasks:
  list_test:
    command: echo "List test"
    description: "This is a test for --list"
"#;
    let script_path = temp_dir.path().join("script.yaml");
    std::fs::write(&script_path, script_content).expect("Failed to write script.yaml");

    let exe = std::env::var("CARGO_BIN_EXE_bodo").expect("CARGO_BIN_EXE_bodo not set");
    let output = Command::new(exe)
        .arg("--list")
        .env("BODO_ROOT_SCRIPT", script_path.to_str().unwrap())
        .env("BODO_NO_WATCH", "1")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute main binary with --list");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Default task"),
        "List output missing default task description"
    );
    assert!(
        stdout.contains("This is a test for --list"),
        "List output missing specific task description"
    );
}
