// tests/basic_integration_tests.rs

use std::fs;
use tempfile::tempdir;

#[test]
fn run_default_task_in_scripts_test() {
    let temp_dir = tempdir().unwrap();
    let script_dir = temp_dir.path().join("scripts").join("test");
    fs::create_dir_all(&script_dir).unwrap();

    let script_content = r#"
name: Test Script
description: Test tasks for the project
default_task:
  command: echo "Running tests"
  description: Default test task
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

    assert!(String::from_utf8_lossy(&output.stdout).contains("Running tests"));
}

#[test]
fn run_build_script_default_task() {
    let temp_dir = tempdir().unwrap();
    let script_dir = temp_dir.path().join("scripts").join("build");
    fs::create_dir_all(&script_dir).unwrap();

    let script_content = r#"
name: Build Script
description: Build tasks for the project
default_task:
  command: echo "Building project"
  description: Default build task
exec_paths:
  - target/debug
  - target/release
env:
  RUST_BACKTRACE: "1"
"#;

    let script_path = script_dir.join("script.yaml");
    fs::write(&script_path, script_content).unwrap();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_bodo"))
        .arg("build")
        .current_dir(&temp_dir)
        .output()
        .unwrap_or_else(|e| panic!("Failed to execute command: {}", e));

    assert!(
        output.status.success(),
        "Unexpected failure.\ncode={:?}\nstdout=```{}```\nstderr=```{}```\ncommand=`cd {:?} && {:?} \"build\"`",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
        temp_dir.path(),
        env!("CARGO_BIN_EXE_bodo"),
    );

    assert!(String::from_utf8_lossy(&output.stdout).contains("Building project"));
}

#[test]
fn run_watch_mode_simulation() {
    let temp_dir = tempdir().unwrap();
    let script_dir = temp_dir.path().join("scripts").join("watch-test");
    fs::create_dir_all(&script_dir).unwrap();

    let script_content = r#"
name: Watch Test Script
description: Watch mode test tasks
default_task:
  command: echo "Watching for changes"
  description: Default watch task
"#;

    let script_path = script_dir.join("script.yaml");
    fs::write(&script_path, script_content).unwrap();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_bodo"))
        .arg("watch-test")
        .current_dir(&temp_dir)
        .output()
        .unwrap_or_else(|e| panic!("Failed to execute command: {}", e));

    assert!(
        output.status.success(),
        "Unexpected failure.\ncode={:?}\nstdout=```{}```\nstderr=```{}```\ncommand=`cd {:?} && {:?} \"watch-test\"`",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
        temp_dir.path(),
        env!("CARGO_BIN_EXE_bodo"),
    );

    assert!(String::from_utf8_lossy(&output.stdout).contains("Watching for changes"));
}
