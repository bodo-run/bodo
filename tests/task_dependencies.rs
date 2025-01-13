use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use tempfile::tempdir;

/// Tests task dependencies as described in the README
#[test]
fn test_task_dependencies() {
    let temp = tempdir().unwrap();
    let project_root = temp.path();
    let dir = project_root.join("scripts").join("test-deps");
    std::fs::create_dir_all(&dir).unwrap();

    // This script has a defaultTask with dependencies on subtasks
    fs::write(
        dir.join("script.yaml"),
        r#"
name: Dependencies Test
defaultTask:
  command: echo "Final task"
  pre_deps:
    - build
    - test
subtasks:
  build:
    command: echo "Building first..."
  test:
    command: echo "Testing second..."
    pre_deps:
      - build
"#,
    )
    .unwrap();

    // Running `bodo test-deps` should execute tasks in correct order
    Command::cargo_bin("bodo")
        .unwrap()
        .current_dir(&project_root)
        .arg("test-deps")
        .assert()
        .success()
        .stdout(contains("Building first..."))
        .stdout(contains("Testing second..."))
        .stdout(contains("Final task"));
}

/// Tests circular dependency detection
#[test]
fn test_circular_dependencies() {
    let temp = tempdir().unwrap();
    let project_root = temp.path();
    let dir = project_root.join("scripts").join("circular-deps");
    std::fs::create_dir_all(&dir).unwrap();

    fs::write(
        dir.join("script.yaml"),
        r#"
name: Circular Dependencies Test
defaultTask:
  command: echo "Main task"
  pre_deps:
    - task-a
subtasks:
  task-a:
    command: echo "Task A"
    pre_deps:
      - task-b
  task-b:
    command: echo "Task B"
    pre_deps:
      - task-a
"#,
    )
    .unwrap();

    // Should fail with circular dependency error
    Command::cargo_bin("bodo")
        .unwrap()
        .current_dir(&project_root)
        .arg("circular-deps")
        .assert()
        .failure()
        .stderr(contains("circular"));
}
