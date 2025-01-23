use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn list_tasks() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("bodo")?;

    cmd.arg("--list")
        .current_dir("tests/fixtures/basic_project")
        .assert()
        .success()
        .stdout(predicate::str::contains("build (from basic)"))
        .stdout(predicate::str::contains("test (from basic)"))
        .stdout(predicate::str::contains("check (from basic)"));

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
