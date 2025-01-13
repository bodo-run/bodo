use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn test_concurrent_tasks() {
    let temp = tempdir().unwrap();
    let project_root = temp.path();

    let dir = project_root.join("scripts/test");
    std::fs::create_dir_all(&dir).unwrap();

    std::fs::write(
        dir.join("script.yaml"),
        r#"
defaultTask:
  concurrently:
    - task: "test1"
    - command: "echo 'Hello from command'"
subtasks:
  test1:
    command: "echo 'Hello from test1'"
"#,
    )
    .unwrap();

    Command::cargo_bin("bodo")
        .unwrap()
        .current_dir(&project_root)
        .arg("test")
        .assert()
        .success()
        .stdout(predicates::str::contains("Hello from command"))
        .stdout(predicates::str::contains("Hello from test1"));
}
