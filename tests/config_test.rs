use bodo::config::{BodoConfig, ConcurrentlyOptions, Dependency, TaskConfig, WatchConfig};
use std::fs;
use tempfile::tempdir;
use validator::Validate;

#[test]
fn test_task_name_validation() {
    let mut task = TaskConfig::default();
    task.command = Some("echo 'test'".to_string());

    // Test reserved names
    task._name_check = Some("watch".to_string());
    assert!(task.validate().is_err());

    task._name_check = Some("default_task".to_string());
    assert!(task.validate().is_err());

    // Test invalid characters
    task._name_check = Some("task/name".to_string());
    assert!(task.validate().is_err());

    task._name_check = Some("task.name".to_string());
    assert!(task.validate().is_err());

    task._name_check = Some("task..name".to_string());
    assert!(task.validate().is_err());

    // Test valid names
    task._name_check = Some("valid-task-name".to_string());
    assert!(task.validate().is_ok());

    task._name_check = Some("task_name_123".to_string());
    assert!(task.validate().is_ok());
}

#[test]
fn test_task_config_validation() {
    let mut task = TaskConfig::default();

    // Test empty task (no command or dependencies)
    assert!(task.validate().is_err());

    // Test valid command
    task.command = Some("echo 'hello'".to_string());
    assert!(task.validate().is_ok());

    // Test valid dependencies
    task.command = None;
    task.pre_deps = vec![Dependency::Command {
        command: "echo 'pre'".to_string(),
    }];
    assert!(task.validate().is_ok());

    // Test invalid timeout
    task.timeout = Some("invalid".to_string());
    assert!(task.validate().is_err());

    // Test valid timeout
    task.timeout = Some("10s".to_string());
    assert!(task.validate().is_ok());
}

#[test]
fn test_watch_config_validation() {
    let mut watch = WatchConfig {
        patterns: vec!["src/**/*.rs".to_string()],
        debounce_ms: 500,
        ignore_patterns: vec![],
        auto_watch: false,
    };

    // Test valid config
    assert!(watch.validate().is_ok());

    // Test empty patterns
    watch.patterns = vec![];
    assert!(watch.validate().is_err());

    // Test invalid debounce
    watch.patterns = vec!["src/**/*.rs".to_string()];
    watch.debounce_ms = 0;
    assert!(watch.validate().is_err());

    watch.debounce_ms = 120_001; // > 2 minutes
    assert!(watch.validate().is_err());

    // Test valid debounce
    watch.debounce_ms = 1000;
    assert!(watch.validate().is_ok());
}

#[test]
fn test_concurrently_options_validation() {
    let mut opts = ConcurrentlyOptions {
        fail_fast: Some(true),
        max_concurrent_tasks: Some(5),
        prefix_color: Some("blue".to_string()),
    };

    // Test valid config
    assert!(opts.validate().is_ok());

    // Test invalid max_concurrent_tasks
    opts.max_concurrent_tasks = Some(0);
    assert!(opts.validate().is_err());

    opts.max_concurrent_tasks = Some(1001);
    assert!(opts.validate().is_err());

    // Test valid max_concurrent_tasks
    opts.max_concurrent_tasks = Some(100);
    assert!(opts.validate().is_ok());
}

#[test]
fn test_bodo_config_validation() {
    let mut config = BodoConfig::default();

    // Test empty config
    assert!(config.validate().is_ok());

    // Test invalid scripts_dirs
    config.scripts_dirs = Some(vec![]);
    assert!(config.validate().is_err());

    // Test valid scripts_dirs
    config.scripts_dirs = Some(vec!["scripts/".to_string()]);
    assert!(config.validate().is_ok());

    // Test task validation
    let mut task = TaskConfig::default();
    task.command = Some("echo 'hello'".to_string());
    config.tasks.insert("test-task".to_string(), task);
    assert!(config.validate().is_ok());
}

#[test]
fn test_script_file_validation() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    // Test invalid task name
    let invalid_script = r#"
tasks:
  watch:  # reserved name
    command: "echo 'test'"
"#;
    fs::write(&script_path, invalid_script).unwrap();
    let config = BodoConfig {
        root_script: Some(script_path.to_str().unwrap().to_string()),
        ..Default::default()
    };
    assert!(bodo::script_loader::ScriptLoader::new()
        .build_graph(config)
        .is_err());

    // Test invalid task config (no command or deps)
    let invalid_script = r#"
tasks:
  test:
    description: "Empty task"
"#;
    fs::write(&script_path, invalid_script).unwrap();
    let config = BodoConfig {
        root_script: Some(script_path.to_str().unwrap().to_string()),
        ..Default::default()
    };
    assert!(bodo::script_loader::ScriptLoader::new()
        .build_graph(config)
        .is_err());

    // Test invalid timeout format
    let invalid_script = r#"
tasks:
  test:
    command: "echo 'test'"
    timeout: "invalid"
"#;
    fs::write(&script_path, invalid_script).unwrap();
    let config = BodoConfig {
        root_script: Some(script_path.to_str().unwrap().to_string()),
        ..Default::default()
    };
    assert!(bodo::script_loader::ScriptLoader::new()
        .build_graph(config)
        .is_err());

    // Test valid script
    let valid_script = r#"
tasks:
  test:
    command: "echo 'test'"
    timeout: "10s"
    description: "A valid task"
"#;
    fs::write(&script_path, valid_script).unwrap();
    let config = BodoConfig {
        root_script: Some(script_path.to_str().unwrap().to_string()),
        ..Default::default()
    };
    assert!(bodo::script_loader::ScriptLoader::new()
        .build_graph(config)
        .is_ok());
}

#[test]
fn test_default_task_validation() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    // Test invalid default task
    let invalid_script = r#"
default_task:
  description: "Invalid default task"
"#;
    fs::write(&script_path, invalid_script).unwrap();
    let config = BodoConfig {
        root_script: Some(script_path.to_str().unwrap().to_string()),
        ..Default::default()
    };
    assert!(bodo::script_loader::ScriptLoader::new()
        .build_graph(config)
        .is_err());

    // Test valid default task
    let valid_script = r#"
default_task:
  command: "echo 'default'"
  description: "Valid default task"
"#;
    fs::write(&script_path, valid_script).unwrap();
    let config = BodoConfig {
        root_script: Some(script_path.to_str().unwrap().to_string()),
        ..Default::default()
    };
    assert!(bodo::script_loader::ScriptLoader::new()
        .build_graph(config)
        .is_ok());
}
