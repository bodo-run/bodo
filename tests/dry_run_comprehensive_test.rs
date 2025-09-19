use std::process::Command;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn test_dry_run_flag_available() {
    let current_exe = std::env::current_exe().expect("Failed to get current exe path");
    let target_dir = current_exe
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("Failed to get target directory");

    let exe_path = target_dir.join("debug").join("bodo");
    #[cfg(windows)]
    let exe_path = exe_path.with_extension("exe");

    assert!(exe_path.exists(), "bodo executable not found at {:?}", exe_path);

    let output = Command::new(exe_path)
        .arg("--help")
        .output()
        .expect("Failed to execute bodo --help");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--dry-run"), "Dry-run flag not found in help output");
    assert!(stdout.contains("show what would be executed without running"), 
           "Dry-run description not found in help output");
}

#[test]
fn test_dry_run_execution() {
    let current_exe = std::env::current_exe().expect("Failed to get current exe path");
    let target_dir = current_exe
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("Failed to get target directory");

    let exe_path = target_dir.join("debug").join("bodo");
    #[cfg(windows)]
    let exe_path = exe_path.with_extension("exe");

    let temp_dir = tempdir().expect("Failed to create temp dir");
    let script_content = r#"
default_task:
  command: echo "Hello from dry-run test!"
  description: "Test task for dry-run functionality"

tasks:
  build:
    command: cargo build --release
    description: "Build the project in release mode"
  
  test:
    command: cargo test --all
    description: "Run all tests"
"#;

    let script_path = temp_dir.path().join("script.yaml");
    std::fs::write(&script_path, script_content).expect("Failed to write script.yaml");

    let root_script_env = script_path.to_string_lossy().into_owned();
    let scripts_dirs_env = temp_dir
        .path()
        .join("scripts_empty")
        .to_string_lossy()
        .into_owned();

    let output = Command::new(&exe_path)
        .arg("--dry-run")
        .env("BODO_ROOT_SCRIPT", &root_script_env)
        .env("BODO_SCRIPTS_DIRS", &scripts_dirs_env)
        .env("BODO_NO_WATCH", "1")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute dry-run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("STDOUT:\n{}", stdout);
    println!("STDERR:\n{}", stderr);

    assert_eq!(output.status.code(), Some(0), "Dry-run should exit with code 0");
    
    // Check for dry-run specific output
    assert!(stdout.contains("🔍 Dry-run mode enabled"), 
           "Should contain dry-run mode message");
    assert!(stdout.contains("📋 Dry-run execution plan:"), 
           "Should contain execution plan header");
    assert!(stdout.contains("Commands will be analyzed but not executed"), 
           "Should contain analysis message");
    assert!(stdout.contains("✅ Dry-run completed successfully"), 
           "Should contain success message");
    assert!(stdout.contains("📝 Task: default"), 
           "Should show the default task");
    assert!(stdout.contains("echo \"Hello from dry-run test!\""), 
           "Should show the command");
    assert!(stdout.contains("⏱️  Estimated total execution time:"), 
           "Should show estimated execution time");
}

#[test]
fn test_dry_run_vs_normal_execution() {
    let current_exe = std::env::current_exe().expect("Failed to get current exe path");
    let target_dir = current_exe
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("Failed to get target directory");

    let exe_path = target_dir.join("debug").join("bodo");
    #[cfg(windows)]
    let exe_path = exe_path.with_extension("exe");

    let temp_dir = tempdir().expect("Failed to create temp dir");
    let script_content = r#"
default_task:
  command: echo "Executed for real!"
  description: "Test task to verify dry-run doesn't execute"
"#;

    let script_path = temp_dir.path().join("script.yaml");
    std::fs::write(&script_path, script_content).expect("Failed to write script.yaml");

    let root_script_env = script_path.to_string_lossy().into_owned();
    let scripts_dirs_env = temp_dir
        .path()
        .join("scripts_empty")
        .to_string_lossy()
        .into_owned();

    // Test dry-run
    let dry_run_output = Command::new(&exe_path)
        .arg("--dry-run")
        .env("BODO_ROOT_SCRIPT", &root_script_env)
        .env("BODO_SCRIPTS_DIRS", &scripts_dirs_env)
        .env("BODO_NO_WATCH", "1")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute dry-run command");

    let dry_run_stdout = String::from_utf8_lossy(&dry_run_output.stdout);

    // Test normal execution
    let normal_output = Command::new(&exe_path)
        .env("BODO_ROOT_SCRIPT", &root_script_env)
        .env("BODO_SCRIPTS_DIRS", &scripts_dirs_env)
        .env("BODO_NO_WATCH", "1")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute normal command");

    let normal_stdout = String::from_utf8_lossy(&normal_output.stdout);

    println!("DRY RUN OUTPUT: {}", dry_run_stdout);
    println!("NORMAL OUTPUT: {}", normal_stdout);

    // Dry-run should show the command but not execute it (no actual output)
    assert!(dry_run_stdout.contains("📋 Dry-run execution plan:"), 
           "Dry-run should show execution plan");
    assert!(dry_run_stdout.contains("✅ Dry-run completed successfully"), 
           "Dry-run should show completion message");
    // The actual command output should NOT appear in dry-run
    assert!(!dry_run_stdout.lines().any(|line| line.trim() == "Executed for real!"), 
           "Dry-run should not show actual command output");
    
    // Normal execution SHOULD execute the command
    assert!(normal_stdout.contains("Executed for real!"), 
           "Normal execution should execute the command");
    
    // Both should exit successfully
    assert_eq!(dry_run_output.status.code(), Some(0));
    assert_eq!(normal_output.status.code(), Some(0));
}

#[test]
fn test_dry_run_with_complex_task() {
    let current_exe = std::env::current_exe().expect("Failed to get current exe path");
    let target_dir = current_exe
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("Failed to get target directory");

    let exe_path = target_dir.join("debug").join("bodo");
    #[cfg(windows)]
    let exe_path = exe_path.with_extension("exe");

    let temp_dir = tempdir().expect("Failed to create temp dir");
    let script_content = r#"
default_task:
  command: echo "Default task"
  description: "Default task"

tasks:
  complex:
    command: "curl -X POST https://api.example.com/data && npm install && cargo build"
    description: "Complex task with multiple operations"
    env:
      BUILD_MODE: "release"
      API_KEY: "secret123"
"#;

    let script_path = temp_dir.path().join("script.yaml");
    std::fs::write(&script_path, script_content).expect("Failed to write script.yaml");

    let root_script_env = script_path.to_string_lossy().into_owned();
    let scripts_dirs_env = temp_dir
        .path()
        .join("scripts_empty")
        .to_string_lossy()
        .into_owned();

    let task_name = format!("{} complex", script_path.to_string_lossy());

    let output = Command::new(&exe_path)
        .arg("--dry-run")
        .arg(&task_name)
        .env("BODO_ROOT_SCRIPT", &root_script_env)
        .env("BODO_SCRIPTS_DIRS", &scripts_dirs_env)
        .env("BODO_NO_WATCH", "1")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute dry-run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert_eq!(output.status.code(), Some(0), "Dry-run should exit with code 0");
    assert!(stdout.contains("📝 Task: complex"), "Should show the complex task");
    assert!(stdout.contains("Environment: 3 vars"), "Should show environment variables count");
    assert!(stdout.contains("curl -X POST"), "Should show the command");
}