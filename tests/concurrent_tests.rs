use std::fs;
use tempfile::tempdir;

#[test]
fn test_concurrent_tasks() {
    let temp_dir = tempdir().unwrap();
    let script_dir = temp_dir.path().join("scripts").join("test");
    fs::create_dir_all(&script_dir).unwrap();

    let script_content = r#"
name: Test Script
description: Test concurrent tasks
default_task:
  command: echo "default task"
  description: Default task
tasks:
  task1:
    command: echo "task1"
    description: Task 1
  task2:
    command: echo "task2"
    description: Task 2
concurrently:
  - task: task1
  - task: task2
"#;

    let script_path = script_dir.join("script.yaml");
    fs::write(&script_path, script_content).unwrap();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_bodo"))
        .arg("test")
        .current_dir(&temp_dir)
        .output()
        .unwrap_or_else(|e| panic!("Failed to execute command: {}", e));

    assert!(
        output.status.success(),
        "Unexpected failure.\ncode={:?}\nstdout=```{}```\nstderr=```{}```\ncommand=`cd {:?} && {:?} \"test\"`",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
        temp_dir.path(),
        env!("CARGO_BIN_EXE_bodo"),
    );
}
