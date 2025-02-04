#!/usr/bin/env rust
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_write_files_integration() {
    // Check if write_files.sh exists in the repository root.
    // Use CARGO_MANIFEST_DIR to get repository root.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let write_files_path = Path::new(&manifest_dir).join("write_files.sh");
    if !write_files_path.exists() {
        eprintln!(
            "Skipping test_write_files_integration because write_files.sh not found at {:?}",
            write_files_path
        );
        return;
    }

    let temp_dir = tempdir().expect("Failed to create temp dir");
    let script_content = r#"
>>>> dummy.txt
Hello, world!
Second line.
>>>> another.txt
Another file content."#;
    let input_path = temp_dir.path().join("input.txt");
    std::fs::write(&input_path, script_content).expect("Failed to write input file");

    let output = Command::new(&write_files_path)
        .arg(input_path.to_str().expect("Input path not valid UTF-8"))
        .current_dir(temp_dir.path())
        .output();

    match output {
        Ok(output) => {
            if !output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!(
                    "write_files.sh failed: stdout: {}, stderr: {}",
                    stdout, stderr
                );
            }
        }
        Err(e) => {
            panic!("Failed to execute write_files.sh: {}", e);
        }
    }

    // Verify that the files were created correctly.
    let dummy_txt = temp_dir.path().join("dummy.txt");
    let another_txt = temp_dir.path().join("another.txt");

    assert!(dummy_txt.exists(), "dummy.txt was not created");
    assert!(another_txt.exists(), "another.txt was not created");

    let dummy_content = std::fs::read_to_string(dummy_txt).expect("Failed to read dummy.txt");
    assert_eq!(dummy_content, "Hello, world!\nSecond line.");
    let another_content = std::fs::read_to_string(another_txt).expect("Failed to read another.txt");
    assert_eq!(another_content, "Another file content.");
}
