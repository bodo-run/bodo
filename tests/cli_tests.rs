use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn list_tasks() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("bodo")?;

    cmd.arg("--list")
        .current_dir("tests/fixtures/basic_project")
        .assert()
        .success()
        .stdout(predicate::str::contains("basic build"))
        .stdout(predicate::str::contains("basic test"))
        .stdout(predicate::str::contains("basic check"));

    Ok(())
}

#[test]
fn run_default_task() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("bodo")?;

    cmd.current_dir("tests/fixtures/basic_project")
        .assert()
        .success()
        .stdout(predicate::str::contains("Running default task"));

    Ok(())
}

#[test]
fn invalid_task_should_fail() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("bodo")?;

    cmd.arg("invalid_task").assert().failure();
    Ok(())
}
