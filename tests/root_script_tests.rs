use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use tempfile::tempdir;

/// Helper to run `bodo` from a given directory
fn run_bodo_in_dir(dir: &std::path::Path, args: &[&str]) -> assert_cmd::assert::Assert {
    Command::cargo_bin("bodo")
        .expect("bodo binary not found")
        .current_dir(dir)
        .args(args)
        .assert()
}

#[test]
fn test_bodo_no_args_with_root_script() {
    let temp = tempdir().unwrap();
    let project_root = temp.path();

    // Create scripts/script.yaml to simulate a root script
    let scripts_dir = project_root.join("scripts");
    fs::create_dir_all(&scripts_dir).unwrap();
    fs::write(
        scripts_dir.join("script.yaml"),
        r#"
default_task:
  command: echo "Root script is running"
"#,
    )
    .unwrap();

    // Now run `bodo` with no arguments
    let assert = run_bodo_in_dir(project_root, &[]);
    let output = assert.success().get_output().stdout.clone();
    let stdout = String::from_utf8_lossy(&output);

    // Verify that it ran the root script
    assert!(
        stdout.contains("Root script is running"),
        "Expected root script to run"
    );
}

#[test]
fn test_bodo_no_args_without_root_script() {
    let temp = tempdir().unwrap();
    let project_root = temp.path();

    // scripts/script.yaml does NOT exist
    fs::create_dir_all(project_root.join("scripts")).unwrap();

    // Run `bodo` with no arguments
    let assert = run_bodo_in_dir(project_root, &[]);
    // Expect error
    assert.failure().stderr(contains(
        "No task specified and no scripts/script.yaml found",
    ));
}

#[test]
fn test_bodo_named_task_happy_path() {
    let temp = tempdir().unwrap();
    let project_root = temp.path();

    // Create scripts/build/script.yaml
    let build_dir = project_root.join("scripts").join("build");
    fs::create_dir_all(&build_dir).unwrap();
    fs::write(
        build_dir.join("script.yaml"),
        r#"
default_task:
  command: echo "Building project"
"#,
    )
    .unwrap();

    // Now run `bodo build`
    let assert = run_bodo_in_dir(project_root, &["build"]);
    let output = assert.success().get_output().stdout.clone();
    let stdout = String::from_utf8_lossy(&output);

    assert!(
        stdout.contains("Building project"),
        "Expected build script to run"
    );
}

#[test]
fn test_bodo_named_task_does_not_exist() {
    let temp = tempdir().unwrap();
    let project_root = temp.path();

    // We do not create scripts/missing/script.yaml
    fs::create_dir_all(project_root.join("scripts")).unwrap();

    // Now run `bodo missing`
    let assert = run_bodo_in_dir(project_root, &["missing"]);
    assert.failure().stderr(contains("not found"));
}

#[test]
fn test_bodo_subtask_in_named_task() {
    let temp = tempdir().unwrap();
    let project_root = temp.path();

    // Create scripts/test/script.yaml
    let test_dir = project_root.join("scripts").join("test");
    fs::create_dir_all(&test_dir).unwrap();
    fs::write(
        test_dir.join("script.yaml"),
        r#"
default_task:
  command: echo "Default test task"

tasks:
  fast:
    command: echo "Running fast subtask"
"#,
    )
    .unwrap();

    // Run `bodo test fast`
    let assert = run_bodo_in_dir(project_root, &["test", "fast"]);
    let output = assert.success().get_output().stdout.clone();
    let stdout = String::from_utf8_lossy(&output);

    assert!(
        stdout.contains("Running fast subtask"),
        "Expected 'fast' subtask to run"
    );
}

#[test]
fn test_bodo_watch_mode_with_root_script() {
    let temp = tempdir().unwrap();
    let project_root = temp.path();

    // Create scripts/script.yaml
    let scripts_dir = project_root.join("scripts");
    fs::create_dir_all(&scripts_dir).unwrap();
    fs::write(
        scripts_dir.join("script.yaml"),
        r#"
default_task:
  command: echo "Root script is running"
"#,
    )
    .unwrap();

    // Simulate watch mode `bodo -w`
    let assert = run_bodo_in_dir(project_root, &["-w"]);
    // Since watch mode is not fully implemented, just check it succeeds
    assert.success();
}

#[test]
fn test_bodo_watch_mode_with_named_script() {
    let temp = tempdir().unwrap();
    let project_root = temp.path();

    // Create scripts/dev/script.yaml
    let dev_dir = project_root.join("scripts").join("dev");
    fs::create_dir_all(&dev_dir).unwrap();
    fs::write(
        dev_dir.join("script.yaml"),
        r#"
default_task:
  command: echo "Development watch in progress"
"#,
    )
    .unwrap();

    // Run `bodo -w dev`
    let assert = run_bodo_in_dir(project_root, &["-w", "dev"]);
    // Since watch mode is not fully implemented, just check it succeeds
    assert.success();
}

#[test]
fn test_bodo_list_flag_shows_scripts() {
    let temp = tempdir().unwrap();
    let project_root = temp.path();

    // Create scripts/test/script.yaml
    let test_dir = project_root.join("scripts").join("test");
    fs::create_dir_all(&test_dir).unwrap();
    fs::write(
        test_dir.join("script.yaml"),
        r#"
default_task:
  command: echo "Test script"
"#,
    )
    .unwrap();

    // Run `bodo --list`
    let assert = run_bodo_in_dir(project_root, &["--list"]);
    let output = assert.success().get_output().stdout.clone();
    let stdout = String::from_utf8_lossy(&output);

    assert!(
        stdout.contains("test"),
        "Expected `test` directory to appear in the listed tasks"
    );
}

#[test]
fn test_fail_fast_logic() {
    let temp = tempdir().unwrap();
    let project_root = temp.path();

    let script_dir = project_root.join("scripts").join("concurrent");
    fs::create_dir_all(&script_dir).unwrap();
    fs::write(
        script_dir.join("script.yaml"),
        r#"
default_task:
  concurrently_options:
    fail_fast: true
  concurrently:
    - command: "echo Start1 && sleep 1 && exit 1"
    - command: "echo Start2 && sleep 5 && echo 'You should never see me'"
"#,
    )
    .unwrap();

    let assert = run_bodo_in_dir(project_root, &["concurrent"]);
    let output = assert.get_output().clone();

    assert.failure();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Start1"));
    assert!(!stdout.contains("You should never see me"));
}

#[test]
fn test_environment_variables_via_script_config() {
    let temp = tempdir().unwrap();
    let project_root = temp.path();

    let env_dir = project_root.join("scripts").join("env");
    fs::create_dir_all(&env_dir).unwrap();
    fs::write(
        env_dir.join("script.yaml"),
        r#"
default_task:
  env:
    MY_VAR: "Hello"
  command: "echo $MY_VAR"
"#,
    )
    .unwrap();

    let assert = run_bodo_in_dir(project_root, &["env"]);
    let output = assert.success().get_output().stdout.clone();
    let stdout = String::from_utf8_lossy(&output);

    assert!(stdout.contains("Hello"), "Should see MY_VAR from script");
}

#[test]
fn test_subtask_dependencies() {
    let temp = tempdir().unwrap();
    let project_root = temp.path();

    let tasks_dir = project_root.join("scripts").join("deps");
    fs::create_dir_all(&tasks_dir).unwrap();

    fs::write(
        tasks_dir.join("script.yaml"),
        r#"
default_task:
  command: echo "Running default"
  pre_deps:
    - task: compile
tasks:
  compile:
    command: echo "Compiling..."
    pre_deps:
      - command: "echo 'Pre-compile command'"
"#,
    )
    .unwrap();

    let assert = run_bodo_in_dir(project_root, &["deps"]);
    let output = assert.success().get_output().stdout.clone();
    let stdout = String::from_utf8_lossy(&output);

    assert!(stdout.contains("Pre-compile command"));
    assert!(stdout.contains("Compiling..."));
    assert!(stdout.contains("Running default"));
}

#[test]
fn test_circular_dependency_fails() {
    let temp = tempdir().unwrap();
    let project_root = temp.path();

    let tasks_dir = project_root.join("scripts").join("circular");
    fs::create_dir_all(&tasks_dir).unwrap();

    fs::write(
        tasks_dir.join("script.yaml"),
        r#"
default_task:
  command: echo "This never runs"
  pre_deps:
    - task: subA

tasks:
  subA:
    command: echo "subA"
    pre_deps:
      - task: subB

  subB:
    command: echo "subB"
    pre_deps:
      - task: subA
"#,
    )
    .unwrap();

    let assert = run_bodo_in_dir(project_root, &["circular"]);
    assert
        .failure()
        .stderr(contains("Circular dependency detected"));
}
