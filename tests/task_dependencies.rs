use std::fs;
use tempfile::tempdir;

#[test]
fn test_task_dependencies() {
    let temp_dir = tempdir().unwrap();
    let script_dir = temp_dir.path().join("scripts").join("test-deps");
    fs::create_dir_all(&script_dir).unwrap();

    let script_content = r#"
name: Test Dependencies Script
description: Test task dependencies
default_task:
  command: echo "Default task"
  description: Default task
  pre_deps:
    - task: setup
    - task: build
tasks:
  setup:
    command: echo "Setting up..."
    description: Setup task
  build:
    command: echo "Building..."
    description: Build task
    pre_deps:
      - task: setup
"#;

    let script_path = script_dir.join("script.yaml");
    fs::write(&script_path, script_content).unwrap();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_bodo"))
        .arg("test-deps")
        .current_dir(&temp_dir)
        .output()
        .unwrap_or_else(|e| panic!("Failed to execute command: {}", e));

    assert!(
        output.status.success(),
        "Unexpected failure.\ncode={:?}\nstdout=```{}```\nstderr=```{}```\ncommand=`cd {:?} && {:?} \"test-deps\"`",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
        temp_dir.path(),
        env!("CARGO_BIN_EXE_bodo"),
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Setting up..."));
    assert!(stdout.contains("Building..."));
    assert!(stdout.contains("Default task"));
}

#[test]
fn test_circular_dependencies() {
    let temp_dir = tempdir().unwrap();
    let script_dir = temp_dir.path().join("scripts").join("circular-deps");
    fs::create_dir_all(&script_dir).unwrap();

    let script_content = r#"
name: Circular Dependencies Script
description: Test circular dependencies
default_task:
  command: echo "Default task"
  description: Default task
  pre_deps:
    - task: task1
tasks:
  task1:
    command: echo "Task 1"
    description: Task 1
    pre_deps:
      - task: task2
  task2:
    command: echo "Task 2"
    description: Task 2
    pre_deps:
      - task: task1
"#;

    let script_path = script_dir.join("script.yaml");
    fs::write(&script_path, script_content).unwrap();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_bodo"))
        .arg("circular-deps")
        .current_dir(&temp_dir)
        .output()
        .unwrap_or_else(|e| panic!("Failed to execute command: {}", e));

    assert!(
        !output.status.success(),
        "Expected failure due to circular dependencies"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Circular dependency detected"));
}
