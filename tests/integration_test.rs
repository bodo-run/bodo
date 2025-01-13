// tests/basic_integration_tests.rs

use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn run_default_task_in_scripts_test() {
    // Prepare a temporary project dir
    let temp = tempdir().unwrap();
    let project_root = temp.path();

    // Simulate a "scripts/test/script.yaml" as described in the README
    let test_scripts_dir = project_root.join("scripts/test");
    fs::create_dir_all(&test_scripts_dir).unwrap();
    fs::write(
        test_scripts_dir.join("script.yaml"),
        r#"
name: Test Script
description: Test tasks for the project
defaultTask:
  command: "echo 'Hello from testTask'"
"#,
    )
    .unwrap();

    // Run "bodo test" and expect it to output the "Hello from testTask" line
    Command::cargo_bin("bodo")
        .unwrap()
        .current_dir(&project_root)
        .arg("test") // per README: "bodo <subdirectory>"
        .assert()
        .success()
        .stdout(predicates::str::contains("Hello from testTask"));
}

#[test]
fn run_build_script_default_task() {
    let temp = tempdir().unwrap();
    let project_root = temp.path();

    // Simulate a "scripts/build/script.yaml" file
    let build_scripts_dir = project_root.join("scripts/build");
    fs::create_dir_all(&build_scripts_dir).unwrap();
    fs::write(
        build_scripts_dir.join("script.yaml"),
        r#"
name: Build Script
description: Build tasks for the project
exec_paths:
  - target/debug
env:
  RUST_BACKTRACE: "1"
defaultTask:
  command: echo "Building as described in README"
"#,
    )
    .unwrap();

    // Invoke "bodo build" and expect it to echo something
    Command::cargo_bin("bodo")
        .unwrap()
        .current_dir(&project_root)
        .arg("build")
        .assert()
        .success()
        .stdout(predicates::str::contains("Building as described in README"));
}

#[test]
fn run_watch_mode_simulation() {
    // Not fully testing file watch logic, just ensuring the command runs
    let temp = tempdir().unwrap();
    let project_root = temp.path();

    // Create a minimal watchable script
    let watch_scripts_dir = project_root.join("scripts/watch-test");
    fs::create_dir_all(&watch_scripts_dir).unwrap();
    fs::write(
        watch_scripts_dir.join("script.yaml"),
        r#"
name: Watch Script
description: Watch mode test
defaultTask:
  command: echo "Watch mode run"
watch:
  patterns:
    - "src/**/*.rs"
"#,
    )
    .unwrap();

    // "bodo watch <subdirectory>" usage
    Command::cargo_bin("bodo")
        .unwrap()
        .current_dir(&project_root)
        .args(&["watch-test"])
        .assert()
        // For a real watch test, we'd need more advanced setup; 
        // for now, just confirm it prints something
        .success()
        .stdout(predicates::str::contains("Watch mode run"));
}