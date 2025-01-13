use std::fs;
use tempfile::tempdir;

#[test]
fn test_subtask_run() {
    let temp_dir = tempdir().unwrap();
    let script_dir = temp_dir.path().join("scripts").join("build");
    fs::create_dir_all(&script_dir).unwrap();

    let script_content = r#"
name: Build Script
description: Build tasks for the project
default_task:
  command: echo "Default build task"
  description: Default build task
tasks:
  compile:
    command: echo "Compiling project"
    description: Compile the project
  test:
    command: echo "Running tests"
    description: Run the test suite
"#;

    let script_path = script_dir.join("script.yaml");
    fs::write(&script_path, script_content).unwrap();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_bodo"))
        .args(&["build", "compile"])
        .current_dir(&temp_dir)
        .output()
        .unwrap_or_else(|e| panic!("Failed to execute command: {}", e));

    assert!(
        output.status.success(),
        "Unexpected failure.\ncode={:?}\nstdout=```{}```\nstderr=```{}```\ncommand=`cd {:?} && {:?} \"build\" \"compile\"`",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
        temp_dir.path(),
        env!("CARGO_BIN_EXE_bodo"),
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Compiling project"));
}
