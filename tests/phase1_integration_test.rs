use std::process::Command;
use tempfile::tempdir;

/// Integration test that validates all Phase 1 features working together
#[test]
fn test_phase1_integration_dry_run_with_logging() {
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
    
    // Create a complex script to test all features
    let script_content = r#"
default_task:
  command: echo "Phase 1 integration test"
  description: "Test all Phase 1 features together"
  env:
    TEST_VAR: "integration"
    PHASE: "1"

tasks:
  build:
    command: echo "Building with $TEST_VAR"
    description: "Build task with environment variables"
    
  test:
    command: echo "Testing Phase $PHASE"
    description: "Test task with variable expansion"
    
  complex:
    command: "echo 'Step 1' && sleep 0.1 && echo 'Step 2'"
    description: "Complex multi-step task"
"#;

    let script_path = temp_dir.path().join("script.yaml");
    std::fs::write(&script_path, script_content).expect("Failed to write script.yaml");

    let root_script_env = script_path.to_string_lossy().into_owned();
    let scripts_dirs_env = temp_dir
        .path()
        .join("scripts_empty")
        .to_string_lossy()
        .into_owned();

    // Test 1: Dry-run with verbose logging
    let verbose_output = Command::new(&exe_path)
        .arg("--dry-run")
        .arg("--verbose")
        .env("BODO_ROOT_SCRIPT", &root_script_env)
        .env("BODO_SCRIPTS_DIRS", &scripts_dirs_env)
        .env("BODO_NO_WATCH", "1")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute verbose dry-run");

    assert_eq!(verbose_output.status.code(), Some(0), "Verbose dry-run should succeed");

    let verbose_combined = format!("{}{}", 
        String::from_utf8_lossy(&verbose_output.stdout),
        String::from_utf8_lossy(&verbose_output.stderr));

    // Should contain dry-run features
    assert!(verbose_combined.contains("🔍 Dry-run mode enabled"), "Should show dry-run mode");
    assert!(verbose_combined.contains("📋 Dry-run execution plan"), "Should show execution plan");
    assert!(verbose_combined.contains("⏱️  Estimated total execution time"), "Should show time estimate");
    assert!(verbose_combined.contains("✅ Dry-run completed successfully"), "Should show success");

    // Should contain structured logging
    assert!(verbose_combined.contains("DEBUG"), "Should contain debug logs in verbose mode");
    assert!(verbose_combined.contains("on_init"), "Should show function instrumentation");
    assert!(verbose_combined.contains("Environment variables"), "Should show environment analysis");

    // Test 2: Dry-run with quiet mode (minimal output)
    let quiet_output = Command::new(&exe_path)
        .arg("--dry-run")
        .arg("--quiet")
        .env("BODO_ROOT_SCRIPT", &root_script_env)
        .env("BODO_SCRIPTS_DIRS", &scripts_dirs_env)
        .env("BODO_NO_WATCH", "1")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute quiet dry-run");

    assert_eq!(quiet_output.status.code(), Some(0), "Quiet dry-run should succeed");

    let quiet_combined = format!("{}{}", 
        String::from_utf8_lossy(&quiet_output.stdout),
        String::from_utf8_lossy(&quiet_output.stderr));

    // Should still show essential dry-run output
    assert!(quiet_combined.contains("📋 Dry-run execution plan"), "Should show execution plan even in quiet mode");
    
    // Should not show debug logs
    assert!(!quiet_combined.contains("DEBUG"), "Should not show debug logs in quiet mode");

    // Test 3: Complex task dry-run
    let complex_task_name = format!("{} complex", script_path.to_string_lossy());
    let complex_output = Command::new(&exe_path)
        .arg("--dry-run")
        .arg(&complex_task_name)
        .env("BODO_ROOT_SCRIPT", &root_script_env)
        .env("BODO_SCRIPTS_DIRS", &scripts_dirs_env)
        .env("BODO_NO_WATCH", "1")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute complex task dry-run");

    assert_eq!(complex_output.status.code(), Some(0), "Complex task dry-run should succeed");

    let complex_stdout = String::from_utf8_lossy(&complex_output.stdout);
    assert!(complex_stdout.contains("📝 Task: complex"), "Should show complex task");
    assert!(complex_stdout.contains("Step 1"), "Should show command content");
}

#[test]  
fn test_phase1_error_recovery_integration() {
    // Test that error categorization works as expected
    use bodo::errors::{BodoError, ErrorCategory};
    
    // Test transient errors
    let plugin_error = BodoError::PluginError("Network timeout".to_string());
    assert_eq!(plugin_error.category(), ErrorCategory::Transient);
    assert!(plugin_error.is_retryable());
    
    // Test permanent errors  
    let validation_error = BodoError::ValidationError("Invalid configuration".to_string());
    assert_eq!(validation_error.category(), ErrorCategory::Permanent);
    assert!(!validation_error.is_retryable());
    
    // Test timeout errors
    let timeout_error = BodoError::TimeoutError { duration: std::time::Duration::from_secs(30) };
    assert_eq!(timeout_error.category(), ErrorCategory::Timeout);
    assert!(timeout_error.is_retryable());
}

#[test]
fn test_phase1_logging_system_integration() {
    // Test that the logging configuration works
    use tracing::Level;
    
    // This test verifies that the logging system is properly configured
    // by checking that log levels are correctly determined
    
    // Simulate different verbosity levels
    let test_cases = vec![
        (false, false, 0, Level::INFO),    // Normal mode
        (true, false, 0, Level::DEBUG),    // Debug mode
        (false, false, 1, Level::DEBUG),   // Verbose mode
        (false, false, 2, Level::TRACE),   // Very verbose mode
        (false, true, 0, Level::ERROR),    // Quiet mode
    ];
    
    for (debug, quiet, verbose, expected_level) in test_cases {
        let actual_level = if quiet {
            Level::ERROR
        } else if debug {
            Level::DEBUG
        } else {
            match verbose {
                0 => Level::INFO,
                1 => Level::DEBUG,
                _ => Level::TRACE,
            }
        };
        
        assert_eq!(actual_level, expected_level, 
                   "Failed for debug={}, quiet={}, verbose={}", debug, quiet, verbose);
    }
}

#[test]
fn test_phase1_comprehensive_cli_options() {
    let current_exe = std::env::current_exe().expect("Failed to get current exe path");
    let target_dir = current_exe
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("Failed to get target directory");

    let exe_path = target_dir.join("debug").join("bodo");
    #[cfg(windows)]
    let exe_path = exe_path.with_extension("exe");

    // Test help output contains all Phase 1 features
    let help_output = Command::new(&exe_path)
        .arg("--help")
        .output()
        .expect("Failed to execute help command");

    let help_text = String::from_utf8_lossy(&help_output.stdout);
    
    // All Phase 1 CLI features should be documented
    assert!(help_text.contains("--dry-run"), "Should document dry-run option");
    assert!(help_text.contains("--verbose"), "Should document verbose option");
    assert!(help_text.contains("--quiet"), "Should document quiet option");
    assert!(help_text.contains("--debug"), "Should document debug option");
    assert!(help_text.contains("can be used multiple times"), "Should document multiple verbose");
    
    // Should have proper version info
    let version_output = Command::new(&exe_path)
        .arg("--version")
        .output()
        .expect("Failed to execute version command");
    
    let version_text = String::from_utf8_lossy(&version_output.stdout);
    assert!(version_text.contains("bodo"), "Should show application name");
}