use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use tempfile::tempdir;

/// Tests whether running `bodo <subdirectory> <subtask>` correctly executes a subtask
#[test]
fn test_subtask_run() {
    let temp = tempdir().unwrap();
    let project_root = temp.path();

    let dir = project_root.join("scripts").join("build");
    std::fs::create_dir_all(&dir).unwrap();

    fs::write(
        dir.join("script.yaml"),
        r#"
name: Build Script
description: Subtask test
exec_paths:
  - target/debug
env:
  RUST_BACKTRACE: "1"
defaultTask:
  command: echo "Default build task"
subtasks:
  compile:
    command: echo "Compiling..."
  clean:
    command: echo "Cleaning build..."
"#,
    )
    .unwrap();

    // Run subtask "compile"
    Command::cargo_bin("bodo")
        .unwrap()
        .current_dir(&project_root)
        .args(&["build", "compile"])
        .assert()
        .success()
        .stdout(contains("Compiling..."));

    // Run subtask "clean"
    Command::cargo_bin("bodo")
        .unwrap()
        .current_dir(&project_root)
        .args(&["build", "clean"])
        .assert()
        .success()
        .stdout(contains("Cleaning build..."));
}
