// tests/main_test.rs

use std::process::Command;

#[test]
fn test_bodo_default() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "default"])
        .env("RUST_LOG", "info")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("STDOUT:\n{}", stdout);
    println!("STDERR:\n{}", stderr);
    let output_combined = format!("{}{}", stdout, stderr);
    assert!(
        output_combined.contains("Hello from Bodo root!"),
        "Output does not contain expected message 'Hello from Bodo root!'"
    );
}

#[test]
fn test_bodo_list() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--list"])
        .env("RUST_LOG", "info")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("STDOUT:\n{}", stdout);
    println!("STDERR:\n{}", stderr);
    let output_combined = format!("{}{}", stdout, stderr);
    assert!(
        output_combined.contains("Default greeting when running"),
        "Output does not contain expected task descriptions"
    );
}
