use std::fs;
use std::process::Command;
use tempfile::tempdir;

fn cargo_run_path() -> String {
    env!("CARGO_BIN_EXE_bodo").to_string()
}

#[test]
fn test_single_task_output_formatting() {
    let temp_dir = tempdir().unwrap();
    let script_dir = temp_dir.path().join("scripts");
    fs::create_dir_all(&script_dir).unwrap();
    let script_path = script_dir.join("script.yaml");

    let script_content = r#"
name: Test Script
description: Test output formatting
default_task:
  command: echo "Hello from task"
  output:
    prefix: "Custom"
    color: BrightBlue
"#;

    fs::write(&script_path, script_content).unwrap();

    println!("Script path: {:?}", script_path);
    println!("Current dir: {:?}", temp_dir.path());

    let output = Command::new(cargo_run_path())
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    println!("Exit status: {:?}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("[Custom]"));
    assert!(stdout.contains("Hello from task"));
}

#[test]
fn test_concurrent_output_formatting() {
    let temp_dir = tempdir().unwrap();
    let script_dir = temp_dir.path().join("scripts");
    fs::create_dir_all(&script_dir).unwrap();
    let script_path = script_dir.join("script.yaml");

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

    fs::write(&script_path, script_content).unwrap();

    println!("Script path: {:?}", script_path);
    println!("Current dir: {:?}", temp_dir.path());

    let output = Command::new(cargo_run_path())
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    println!("Exit status: {:?}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[First]"));
    assert!(stdout.contains("[Second]"));
}

#[test]
fn test_fallback_output_formatting() {
    let temp_dir = tempdir().unwrap();
    let script_dir = temp_dir.path().join("scripts");
    fs::create_dir_all(&script_dir).unwrap();
    let script_path = script_dir.join("script.yaml");

    let script_content = r#"
name: Test Script
description: Test fallback output formatting
tasks:
  subtask:
    command: echo "From subtask"
default_task:
  concurrently:
    - task: subtask
    - command: echo "Direct command"
"#;

    fs::write(&script_path, script_content).unwrap();

    println!("Script path: {:?}", script_path);
    println!("Current dir: {:?}", temp_dir.path());

    let output = Command::new(cargo_run_path())
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    println!("Exit status: {:?}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Actual stdout: {}", stdout);
    assert!(stdout.contains("[.:subtask] echo \"From subtask\""));
    assert!(stdout.contains("[.:command2] echo \"Direct command\""));
}
