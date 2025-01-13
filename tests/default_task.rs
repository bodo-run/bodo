use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use tempfile::tempdir;

/// Tests whether running `bodo <subdirectory>` executes the default task
#[test]
fn test_default_task_on_subdirectory() {
    let temp = tempdir().unwrap();
    let project_root = temp.path();

    // Create a script dir following the README's structure
    let dir = project_root.join("scripts").join("my-task-group");
    std::fs::create_dir_all(&dir).unwrap();

    // Write a minimal script.yaml with a defaultTask
    fs::write(
        dir.join("script.yaml"),
        r#"
name: My Task
description: Just a test
exec_paths:
  - node_modules/.bin
env:
  MY_VAR: "Hello from default task"
defaultTask:
  command: echo "Executing the default task"
"#,
    )
    .unwrap();

    // Run `bodo my-task-group`
    Command::cargo_bin("bodo")
        .unwrap()
        .current_dir(&project_root)
        .arg("my-task-group")
        .assert()
        .success()
        .stdout(contains("Executing the default task"));
}

/// Tests whether environment variables from a script.yaml are recognized
#[test]
fn test_env_variables_in_default_task() {
    let temp = tempdir().unwrap();
    let project_root = temp.path();

    let dir = project_root.join("scripts").join("env-test");
    std::fs::create_dir_all(&dir).unwrap();

    fs::write(
        dir.join("script.yaml"),
        r#"
name: Env Test
description: Checking env
exec_paths:
  - my_fake_bin
env:
  TEST_GREETING: "Hello from BODO"
defaultTask:
  command: printenv TEST_GREETING
"#,
    )
    .unwrap();

    Command::cargo_bin("bodo")
        .unwrap()
        .current_dir(&project_root)
        .arg("env-test")
        .assert()
        .success()
        .stdout(contains("Hello from BODO"));
}
