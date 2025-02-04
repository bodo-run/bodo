use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_write_files_integration() {
    // locate the write_files.sh script in the repository root
    let script_path = Path::new("write_files.sh");
    assert!(script_path.exists(), "write_files.sh does not exist");

    // create a temporary directory to run the script
    let temp_dir = tempdir().unwrap();

    // prepare an input file containing sections for multiple output files
    let input_content = "\
>>>> file1.txt
Hello, file1!
>>>> subdir/file2.txt
Hello, file2!
Second line of file2.
";
    let input_file_path = temp_dir.path().join("input.txt");
    fs::write(&input_file_path, input_content).unwrap();

    // Run the write_files.sh script with the input file in the temporary directory.
    let output = Command::new(script_path)
        .arg(input_file_path.to_str().unwrap())
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute write_files.sh");
    assert!(output.status.success(), "write_files.sh did not succeed");

    // Check that file1.txt and subdir/file2.txt were created with the expected content.
    let file1_path = temp_dir.path().join("file1.txt");
    let file2_path = temp_dir.path().join("subdir").join("file2.txt");
    assert!(file1_path.exists(), "file1.txt was not created");
    assert!(file2_path.exists(), "subdir/file2.txt was not created");

    let file1_content = fs::read_to_string(file1_path).unwrap();
    let file2_content = fs::read_to_string(file2_path).unwrap();

    assert_eq!(file1_content, "Hello, file1!");
    assert_eq!(file2_content, "Hello, file2!\nSecond line of file2.");
}
