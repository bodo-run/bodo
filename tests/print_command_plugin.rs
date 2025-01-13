use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_print_command_plugin() {
    let temp = tempdir().unwrap();
    let project_root = temp.path();

    let dir = project_root.join("scripts/test");
    fs::create_dir_all(&dir).unwrap();

    fs::write(
        dir.join("script.yaml"),
        r#"
name: Print Test
default_task:
  command: "echo 'Hello, BODO!'"
tasks:
  silent_task:
    command: "echo 'This should be silent'"
    silent: true
"#,
    )
    .unwrap();

    let output = Command::cargo_bin("bodo")
        .unwrap()
        .current_dir(&project_root)
        .arg("test")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);

    // Check if the command is printed
    assert!(stdout.contains("> test: echo 'Hello, BODO!'"));
}
