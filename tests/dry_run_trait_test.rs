use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use bodo::plugin::{DryRunnable, ExecutionContext, SideEffect};
use bodo::plugins::execution_plugin::ExecutionPlugin;

#[test]
fn test_execution_plugin_dry_run_trait() {
    let plugin = ExecutionPlugin::new();
    let context = ExecutionContext {
        task_name: "test_task".to_string(),
        working_directory: PathBuf::from("/tmp"),
        environment: HashMap::from([
            ("TEST_VAR".to_string(), "test_value".to_string()),
            ("PATH".to_string(), "/usr/bin".to_string()),
        ]),
    };

    let result = plugin.dry_run(&context);
    assert!(result.is_ok(), "Dry run should succeed");

    let report = result.unwrap();
    assert_eq!(report.working_directory, PathBuf::from("/tmp"));
    assert_eq!(report.environment.len(), 2);
    assert!(report.environment.contains_key("TEST_VAR"));
    assert!(report.environment.contains_key("PATH"));
    assert!(report.estimated_duration.is_some());
    assert!(!report.side_effects.is_empty());
}

#[test]
fn test_side_effect_analysis() {
    let plugin = ExecutionPlugin::new();
    
    // Test file creation detection
    let file_cmd = "touch newfile.txt && echo 'content' > output.log";
    let side_effects = plugin.analyze_side_effects(file_cmd, &Some(PathBuf::from("/tmp")));
    
    let has_file_creation = side_effects.iter().any(|effect| {
        matches!(effect, SideEffect::FileCreation(_))
    });
    assert!(has_file_creation, "Should detect file creation side effects");

    let has_process_spawn = side_effects.iter().any(|effect| {
        matches!(effect, SideEffect::ProcessSpawn(_))
    });
    assert!(has_process_spawn, "Should always detect process spawn");

    // Test network request detection
    let network_cmd = "curl https://api.example.com/data";
    let network_effects = plugin.analyze_side_effects(network_cmd, &None);
    let has_network = network_effects.iter().any(|effect| {
        matches!(effect, SideEffect::NetworkRequest(_))
    });
    assert!(has_network, "Should detect network requests");

    // Test file deletion detection
    let delete_cmd = "rm -rf old_files/";
    let delete_effects = plugin.analyze_side_effects(delete_cmd, &Some(PathBuf::from("/tmp")));
    let has_deletion = delete_effects.iter().any(|effect| {
        matches!(effect, SideEffect::FileDeletion(_))
    });
    assert!(has_deletion, "Should detect file deletions");

    // Test environment change detection
    let env_cmd = "export BUILD_MODE=release";
    let env_effects = plugin.analyze_side_effects(env_cmd, &None);
    let has_env_change = env_effects.iter().any(|effect| {
        matches!(effect, SideEffect::EnvironmentChange(_, _))
    });
    assert!(has_env_change, "Should detect environment changes");
}

#[test]
fn test_duration_estimation() {
    let plugin = ExecutionPlugin::new();

    // Test sleep command estimation
    let sleep_duration = plugin.estimate_duration("sleep 5");
    assert!(sleep_duration.is_some());
    assert_eq!(sleep_duration.unwrap(), Duration::from_secs(1));

    // Test network command estimation
    let curl_duration = plugin.estimate_duration("curl https://api.example.com");
    assert!(curl_duration.is_some());
    assert_eq!(curl_duration.unwrap(), Duration::from_millis(500));

    // Test build command estimation
    let build_duration = plugin.estimate_duration("cargo build --release");
    assert!(build_duration.is_some());
    assert_eq!(build_duration.unwrap(), Duration::from_secs(30));

    // Test npm install estimation
    let npm_duration = plugin.estimate_duration("npm install dependencies");
    assert!(npm_duration.is_some());
    assert_eq!(npm_duration.unwrap(), Duration::from_secs(30));

    // Test default estimation
    let default_duration = plugin.estimate_duration("echo 'hello world'");
    assert!(default_duration.is_some());
    assert_eq!(default_duration.unwrap(), Duration::from_millis(100));
}

#[test]
fn test_env_var_expansion_in_dry_run() {
    let plugin = ExecutionPlugin::new();
    let mut env = HashMap::new();
    env.insert("USER".to_string(), "testuser".to_string());
    env.insert("HOME".to_string(), "/home/testuser".to_string());

    // Test simple variable expansion
    let expanded = plugin.expand_env_vars("echo Hello $USER", &env);
    assert_eq!(expanded, "echo Hello testuser");

    // Test braced variable expansion
    let expanded_braced = plugin.expand_env_vars("cd ${HOME}/projects", &env);
    assert_eq!(expanded_braced, "cd /home/testuser/projects");

    // Test missing variable (should keep as-is)
    let expanded_missing = plugin.expand_env_vars("echo $MISSING_VAR", &env);
    assert_eq!(expanded_missing, "echo $MISSING_VAR");

    // Test escaped dollar
    let expanded_escaped = plugin.expand_env_vars("echo $$USER", &env);
    assert_eq!(expanded_escaped, "echo $USER");
}

#[test]
fn test_dry_run_report_structure() {
    let plugin = ExecutionPlugin::new();
    let context = ExecutionContext {
        task_name: "build".to_string(),
        working_directory: PathBuf::from("/project"),
        environment: HashMap::from([
            ("CC".to_string(), "gcc".to_string()),
            ("CFLAGS".to_string(), "-O2".to_string()),
        ]),
    };

    let report = plugin.dry_run(&context).expect("Dry run should succeed");

    // Verify report structure
    assert!(!report.command.is_empty(), "Command should not be empty");
    assert_eq!(report.working_directory, PathBuf::from("/project"));
    assert_eq!(report.environment.len(), 2);
    assert!(report.environment.contains_key("CC"));
    assert!(report.environment.contains_key("CFLAGS"));
    assert!(report.estimated_duration.is_some(), "Should have duration estimate");
    assert!(!report.side_effects.is_empty(), "Should have side effects");
    assert!(report.dependencies.is_empty(), "Dependencies not implemented yet");

    // Verify side effects contain process spawn
    let has_process_spawn = report.side_effects.iter().any(|effect| {
        matches!(effect, SideEffect::ProcessSpawn(_))
    });
    assert!(has_process_spawn, "Should always include process spawn side effect");
}