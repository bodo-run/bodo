use std::fs;
use tempfile::tempdir;

#[test]
fn test_single_task_output_formatting() {
    let temp_dir = tempdir().unwrap();
    let script_dir = temp_dir.path().join("scripts").join("test");
    fs::create_dir_all(&script_dir).unwrap();

    let script_content = r#"
name: Test Script
description: Test output formatting
default_task:
  command: echo "Hello from task"
  output:
    prefix: "Custom"
    color: BrightBlue
"#;

    let script_path = script_dir.join("script.yaml");
    fs::write(&script_path, script_content).unwrap();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_bodo"))
        .arg("test")
        .current_dir(&temp_dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("[Custom]"));
    assert!(stdout.contains("Hello from task"));
}

#[test]
fn test_concurrent_output_formatting() {
    let temp_dir = tempdir().unwrap();
    let script_dir = temp_dir.path().join("scripts").join("test");
    fs::create_dir_all(&script_dir).unwrap();

    let script_content = r#"
name: Test Script
description: Test concurrent output formatting
default_task:
  concurrently:
    - command: echo "Task 1"
      output:
        prefix: "First"
        color: BrightGreen
    - command: echo "Task 2"
      output:
        prefix: "Second"
        color: BrightYellow
"#;

    let script_path = script_dir.join("script.yaml");
    fs::write(&script_path, script_content).unwrap();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_bodo"))
        .arg("test")
        .current_dir(&temp_dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[First]"));
    assert!(stdout.contains("[Second]"));
}

#[test]
fn test_fallback_output_formatting() {
    let temp_dir = tempdir().unwrap();
    let script_dir = temp_dir.path().join("scripts").join("test");
    fs::create_dir_all(&script_dir).unwrap();

    let script_content = r#"
name: Test Script
description: Test fallback output formatting
default_task:
  concurrently:
    - task: subtask
    - command: echo "Direct command"
tasks:
  subtask:
    command: echo "From subtask"
"#;

    let script_path = script_dir.join("script.yaml");
    fs::write(&script_path, script_content).unwrap();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_bodo"))
        .arg("test")
        .current_dir(&temp_dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Actual stdout: {}", stdout);
    // Check fallback prefixes
    assert!(stdout.contains("[test:subtask]")); // Task reference format
    assert!(stdout.contains("[test:command1]")); // Command format with index
}
