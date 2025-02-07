use std::process::Command;

#[test]
fn test_main_help() {
    // Invoke the binary with --help to cover main.rs paths.
    // The CARGO_BIN_EXE_bodo env variable is automatically set by cargo when testing binaries.
    let exe_var = std::env::var("CARGO_BIN_EXE_bodo");
    if exe_var.is_err() {
        eprintln!("Skipping test_main_help because CARGO_BIN_EXE_bodo is not set");
        return;
    }
    let exe = exe_var.unwrap();
    let output = Command::new(exe)
        .arg("--help")
        .output()
        .expect("Failed to execute main binary with --help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage"));
}
