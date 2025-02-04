use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_main_help_output() {
    // Ensure that the Bodo executable --help prints usage information.
    let exe = std::env::var("CARGO_BIN_EXE_bodo").expect("CARGO_BIN_EXE_bodo not set");
    let output = Command::new(exe)
        .arg("--help")
        .output()
        .expect("Failed to run --help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Usage:"),
        "Help output should contain 'Usage:'"
    );
}

#[test]
fn test_main_dry_run() {
    // Run the Bodo binary in a temporary directory with a simple script file,
    // using the --dry-run flag if supported.
    let exe = std::env::var("CARGO_BIN_EXE_bodo").expect("CARGO_BIN_EXE_bodo not set");
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let script_content = r#"
default_task:
  command: echo "Dry run test"
"#;
    let script_path = temp_dir.path().join("script.yaml");
    std::fs::write(&script_path, script_content).expect("Failed to write script.yaml");

    let output = Command::new(&exe)
        .env("BODO_ROOT_SCRIPT", script_path.to_str().unwrap())
        .env("BODO_NO_WATCH", "1")
        .arg("--dry-run")
        .output()
        .expect("Failed to run Bodo in dry-run mode");
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Expect the output to mention the command but not execute it.
    assert!(
        stdout.contains("echo \"Dry run test\""),
        "Dry-run output missing command"
    );
}
