use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_write_files_script() {
    // Prepare an input file with file sections.
    let input_content = "\
>>>> dummy.txt
Hello, world!
Second line.
>>>> another.txt
Another file content.";
    let temp_dir = tempdir().unwrap();
    let input_path = temp_dir.path().join("input.txt");
    fs::write(&input_path, input_content).expect("Failed to write input file");

    // Ensure that the write_files.sh script exists in the repository root.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let script_path = Path::new(manifest_dir).join("write_files.sh");
    assert!(
        script_path.exists(),
        "write_files.sh does not exist at {}",
        script_path.display()
    );

    // Execute the write_files.sh script with the input file and set the current directory to the temp dir.
    let status = Command::new(&script_path)
        .arg(input_path.to_str().expect("Input path contains invalid UTF-8"))
        .current_dir(temp_dir.path())
        .status()
        .expect("Failed to execute write_files.sh");
    assert!(status.success());

    // Verify that the files were created correctly.
    let dummy_txt = temp_dir.path().join("dummy.txt");
    let another_txt = temp_dir.path().join("another.txt");

    assert!(dummy_txt.exists());
    assert!(another_txt.exists());

    let dummy_content = fs::read_to_string(dummy_txt).expect("Failed to read dummy.txt");
    assert_eq!(dummy_content, "Hello, world!\nSecond line.");
    let another_content = fs::read_to_string(another_txt).expect("Failed to read another.txt");
    assert_eq!(another_content, "Another file content.");
}
