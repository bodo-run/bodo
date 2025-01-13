use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use tempfile::tempdir;

/// Basic watch mode test. The README says `bodo watch <subdirectory>` will re-run tasks on file changes.
#[test]
fn test_watch_mode_basic() {
    let temp = tempdir().unwrap();
    let project_root = temp.path();

    let watch_dir = project_root.join("scripts").join("watch-test");
    fs::create_dir_all(&watch_dir).unwrap();
    fs::write(
        watch_dir.join("script.yaml"),
        r#"
name: Watch Script
defaultTask:
  command: echo "Running watch script..."
watch:
  patterns:
    - "src/**/*.rs"
"#,
    )
    .unwrap();

    // Create a source file to watch
    let src_dir = project_root.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(
        src_dir.join("main.rs"),
        "fn main() { println!(\"Hello\"); }",
    )
    .unwrap();

    Command::cargo_bin("bodo")
        .unwrap()
        .current_dir(&project_root)
        .args(&["watch", "watch-test"])
        .assert()
        .success()
        .stdout(contains("Running watch script..."));
}
