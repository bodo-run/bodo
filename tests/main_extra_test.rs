use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_main_help_output() {
    // Skip test if CARGO_BIN_EXE_bodo is not set.
    if std::env::var("CARGO_BIN_EXE_bodo").is_err() {
        eprintln!("Skipping test_main_help_output because CARGO_BIN_EXE_bodo is not set");
        return;
    }
    // Invoke the binary with --help to cover main.rs paths.
    // The CARGO_BIN_EXE_bodo env variable is automatically set by cargo when testing binaries.
    let exe = std::env::var("CARGO_BIN_EXE_bodo").expect("CARGO_BIN_EXE_bodo not set");
    let output = Command::new(exe)
        .arg("--help")
        .output()
        .expect("Failed to execute main binary with --help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Usage"),
        "Expected help output to contain 'Usage:'"
    );
}

#[test]
fn test_main_dry_run() {
    // Skip test if CARGO_BIN_EXE_bodo is not set.
    if std::env::var("CARGO_BIN_EXE_bodo").is_err() {
        eprintln!("Skipping test_main_dry_run because CARGO_BIN_EXE_bodo is not set");
        return;
    }
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let script_content = r#"
>>>> dummy.txt
Hello, world!
Second line.
>>>> another.txt
Another file content."#;
    let script_path = temp_dir.path().join("script.yaml");
    std::fs::write(&script_path, script_content).expect("Failed to write script.yaml");

    let exe = std::env::var("CARGO_BIN_EXE_bodo").expect("CARGO_BIN_EXE_bodo not set");
    let output = Command::new(exe)
        .env("BODO_ROOT_SCRIPT", script_path.to_str().unwrap())
        .env("BODO_NO_WATCH", "1")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute main binary in dry run mode");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Hello, world!"),
        "Expected output to contain 'Hello, world!'"
    );
}
