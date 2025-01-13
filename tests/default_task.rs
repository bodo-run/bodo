use std::fs;
use tempfile::tempdir;

#[test]
fn test_env_variables_in_default_task() {
    let temp_dir = tempdir().unwrap();
    let script_dir = temp_dir.path().join("scripts").join("env-test");
    fs::create_dir_all(&script_dir).unwrap();

    let script_content = r#"
name: Test Script
description: Test environment variables
default_task:
  command: echo $TEST_VAR
  description: Default task
env:
  TEST_VAR: "Hello from env"
"#;

    let script_path = script_dir.join("script.yaml");
    fs::write(&script_path, script_content).unwrap();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_bodo"))
        .arg("env-test")
        .current_dir(&temp_dir)
        .output()
        .unwrap_or_else(|e| panic!("Failed to execute command: {}", e));

    assert!(
        output.status.success(),
        "Unexpected failure.\ncode={:?}\nstdout=```{}```\nstderr=```{}```\ncommand=`cd {:?} && {:?} \"env-test\"`",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
        temp_dir.path(),
        env!("CARGO_BIN_EXE_bodo"),
    );

    assert!(String::from_utf8_lossy(&output.stdout).contains("Hello from env"));
}

#[test]
fn test_default_task_on_subdirectory() {
    let temp_dir = tempdir().unwrap();
    let script_dir = temp_dir.path().join("scripts").join("my-task-group");
    fs::create_dir_all(&script_dir).unwrap();

    let script_content = r#"
name: Test Script
description: Test default task in subdirectory
default_task:
  command: echo "Hello from subdirectory"
  description: Default task
"#;

    let script_path = script_dir.join("script.yaml");
    fs::write(&script_path, script_content).unwrap();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_bodo"))
        .arg("my-task-group")
        .current_dir(&temp_dir)
        .output()
        .unwrap_or_else(|e| panic!("Failed to execute command: {}", e));

    assert!(
        output.status.success(),
        "Unexpected failure.\ncode={:?}\nstdout=```{}```\nstderr=```{}```\ncommand=`cd {:?} && {:?} \"my-task-group\"`",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
        temp_dir.path(),
        env!("CARGO_BIN_EXE_bodo"),
    );

    assert!(String::from_utf8_lossy(&output.stdout).contains("Hello from subdirectory"));
}
