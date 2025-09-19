use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_verbose_flag_available() {
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
    assert!(stdout.contains("-v, --verbose"), "Verbose flag not found in help output");
    assert!(stdout.contains("-q, --quiet"), "Quiet flag not found in help output");
    assert!(stdout.contains("can be used multiple times"), "Multiple verbose description not found");
}

#[test]
fn test_verbose_logging_levels() {
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
  command: echo "Test logging"
  description: "Test task for logging"
"#;

    let script_path = temp_dir.path().join("script.yaml");
    std::fs::write(&script_path, script_content).expect("Failed to write script.yaml");

    let root_script_env = script_path.to_string_lossy().into_owned();
    let scripts_dirs_env = temp_dir
        .path()
        .join("scripts_empty")
        .to_string_lossy()
        .into_owned();

    // Test normal dry-run (INFO level)
    let output = Command::new(&exe_path)
        .arg("--dry-run")
        .env("BODO_ROOT_SCRIPT", &root_script_env)
        .env("BODO_SCRIPTS_DIRS", &scripts_dirs_env)
        .env("BODO_NO_WATCH", "1")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute normal dry-run");

    let _stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Combine stdout and stderr for checking (tracing might output to either)
    let combined_output = format!("{}{}", _stdout, stderr);
    
    // Should have INFO logs but not DEBUG
    assert!(combined_output.contains("INFO"), "Should contain INFO logs");
    assert!(!combined_output.contains("DEBUG"), "Should not contain DEBUG logs in normal mode");

    // Test verbose dry-run (DEBUG level)
    let verbose_output = Command::new(&exe_path)
        .arg("--dry-run")
        .arg("--verbose")
        .env("BODO_ROOT_SCRIPT", &root_script_env)
        .env("BODO_SCRIPTS_DIRS", &scripts_dirs_env)
        .env("BODO_NO_WATCH", "1")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute verbose dry-run");

    let verbose_combined = format!("{}{}", 
        String::from_utf8_lossy(&verbose_output.stdout),
        String::from_utf8_lossy(&verbose_output.stderr));
    
    // Should have both INFO and DEBUG logs
    assert!(verbose_combined.contains("INFO"), "Should contain INFO logs in verbose mode");
    assert!(verbose_combined.contains("DEBUG"), "Should contain DEBUG logs in verbose mode");
}

#[test]
fn test_quiet_mode_logging() {
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
  command: echo "Test quiet"
  description: "Test task for quiet mode"
"#;

    let script_path = temp_dir.path().join("script.yaml");
    std::fs::write(&script_path, script_content).expect("Failed to write script.yaml");

    let root_script_env = script_path.to_string_lossy().into_owned();
    let scripts_dirs_env = temp_dir
        .path()
        .join("scripts_empty")
        .to_string_lossy()
        .into_owned();

    // Test quiet dry-run (ERROR level only)
    let quiet_output = Command::new(&exe_path)
        .arg("--dry-run")
        .arg("--quiet")
        .env("BODO_ROOT_SCRIPT", &root_script_env)
        .env("BODO_SCRIPTS_DIRS", &scripts_dirs_env)
        .env("BODO_NO_WATCH", "1")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute quiet dry-run");

    let quiet_combined = format!("{}{}", 
        String::from_utf8_lossy(&quiet_output.stdout),
        String::from_utf8_lossy(&quiet_output.stderr));
    
    // Should not have INFO or DEBUG logs in quiet mode
    assert!(!quiet_combined.contains("INFO"), "Should not contain INFO logs in quiet mode");
    assert!(!quiet_combined.contains("DEBUG"), "Should not contain DEBUG logs in quiet mode");
    
    // But stdout should still have the dry-run output
    let quiet_stdout = String::from_utf8_lossy(&quiet_output.stdout);
    assert!(quiet_stdout.contains("📋 Dry-run execution plan:"), "Should still show dry-run output");
}

#[test]
fn test_tracing_instrumentation() {
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
  command: echo "Test instrumentation"
  description: "Test task for tracing instrumentation"
"#;

    let script_path = temp_dir.path().join("script.yaml");
    std::fs::write(&script_path, script_content).expect("Failed to write script.yaml");

    let root_script_env = script_path.to_string_lossy().into_owned();
    let scripts_dirs_env = temp_dir
        .path()
        .join("scripts_empty")
        .to_string_lossy()
        .into_owned();

    // Test with maximum verbosity to see instrumentation
    let output = Command::new(&exe_path)
        .arg("--dry-run")
        .arg("-vv") // Double verbose
        .env("BODO_ROOT_SCRIPT", &root_script_env)
        .env("BODO_SCRIPTS_DIRS", &scripts_dirs_env)
        .env("BODO_NO_WATCH", "1")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute instrumented dry-run");

    let combined_output = format!("{}{}", 
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr));
    
    // Should contain tracing spans with function names
    assert!(combined_output.contains("on_init"), "Should contain instrumented function spans");
    assert!(combined_output.contains("perform_dry_run"), "Should contain dry-run function spans");
    assert!(combined_output.contains("analyze_task_tree"), "Should contain analysis function spans");
    
    // With -vv should include file names and line numbers
    assert!(combined_output.contains("src/plugins/execution_plugin.rs"), "Should include file names with -vv");
}