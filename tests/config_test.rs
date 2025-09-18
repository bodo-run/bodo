// tests/config_test.rs

use bodo::config::{BodoConfig, Dependency, TaskConfig, WatchConfig};
use validator::Validate;
use validator::ValidationErrors;

#[allow(clippy::field_reassign_with_default)]
#[test]
fn test_validate_task_name_reserved() {
    let mut config = TaskConfig::default();
    config._name_check = Some("default_task".to_string());
    config.command = Some("echo 'test'".to_string()); // Add command to pass validation
    let result = config.validate();
    assert!(matches!(result, Err(ValidationErrors { .. })));
}

#[allow(clippy::field_reassign_with_default)]
#[test]
fn test_validate_task_name_valid() {
    let mut config = TaskConfig::default();
    config._name_check = Some("valid_task_name".to_string());
    config.command = Some("echo 'test'".to_string()); // Add command to pass validation
    let result = config.validate();
    assert!(
        result.is_ok(),
        "Expected validation to pass for valid task name"
    );
}

#[allow(clippy::field_reassign_with_default)]
#[test]
fn test_validate_task_name_invalid_characters() {
    let mut config = TaskConfig::default();
    config._name_check = Some("invalid/task/name".to_string());
    config.command = Some("echo 'test'".to_string()); // Add command
    let result = config.validate();
    assert!(matches!(result, Err(ValidationErrors { .. })));

    let mut config = TaskConfig::default();
    config._name_check = Some("invalid..name".to_string());
    config.command = Some("echo 'test'".to_string()); // Add command
    let result = config.validate();
    assert!(matches!(result, Err(ValidationErrors { .. })));

    let mut config = TaskConfig::default();
    config._name_check = Some("invalid.name".to_string());
    config.command = Some("echo 'test'".to_string()); // Add command
    let result = config.validate();
    assert!(matches!(result, Err(ValidationErrors { .. })));
}

#[allow(clippy::field_reassign_with_default)]
#[test]
fn test_validate_task_name_invalid_length() {
    let mut config = TaskConfig::default();
    config._name_check = Some("".to_string());
    config.command = Some("echo 'test'".to_string()); // Add command
    let result = config.validate();
    assert!(matches!(result, Err(ValidationErrors { .. })));

    let long_name = "a".repeat(101);
    let mut config = TaskConfig::default();
    config._name_check = Some(long_name);
    config.command = Some("echo 'test'".to_string()); // Add command
    let result = config.validate();
    assert!(matches!(result, Err(ValidationErrors { .. })));
}

#[test]
fn test_validate_task_config_no_command_no_deps() {
    let task_config = TaskConfig {
        command: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        ..Default::default()
    };
    let result = task_config.validate();
    assert!(matches!(result, Err(ValidationErrors { .. })));
}

#[test]
fn test_validate_task_config_with_deps_no_command() {
    let task_config = TaskConfig {
        command: None,
        pre_deps: vec![Dependency::Task {
            task: "some_task".to_string(),
        }],
        ..Default::default()
    };
    let result = task_config.validate();
    assert!(
        result.is_ok(),
        "Expected validation to pass with dependencies"
    );
}

#[test]
fn test_validate_task_config_invalid_timeout() {
    let task_config = TaskConfig {
        command: Some("echo 'Hello'".to_string()),
        timeout: Some("invalid_duration".to_string()),
        ..Default::default()
    };
    let result = task_config.validate();
    assert!(matches!(result, Err(ValidationErrors { .. })));
}

#[test]
fn test_validate_task_config_valid_timeout() {
    let task_config = TaskConfig {
        command: Some("echo 'Hello'".to_string()),
        timeout: Some("30s".to_string()),
        ..Default::default()
    };
    let result = task_config.validate();
    assert!(
        result.is_ok(),
        "Expected validation to pass with valid timeout"
    );
}

#[test]
fn test_task_config_with_all_fields() {
    let task_config = TaskConfig {
        description: Some("A full task".to_string()),
        command: Some("echo 'Running task'".to_string()),
        cwd: Some("/tmp".to_string()),
        env: [("VAR1".to_string(), "value1".to_string())]
            .iter()
            .cloned()
            .collect(),
        pre_deps: vec![Dependency::Task {
            task: "pre_task".to_string(),
        }],
        post_deps: vec![Dependency::Command {
            command: "echo 'Post'".to_string(),
        }],
        watch: Some(WatchConfig {
            patterns: vec!["src/**/*.rs".to_string()],
            debounce_ms: 500,
            ignore_patterns: vec![],
            auto_watch: false,
        }),
        timeout: Some("1m".to_string()),
        exec_paths: vec!["/usr/local/bin".to_string()],
        arguments: vec![],
        concurrently_options: Default::default(),
        concurrently: vec![],
        _name_check: Some("full_task".to_string()),
    };
    let result = task_config.validate();
    assert!(
        result.is_ok(),
        "Expected validation to pass for full task config"
    );
}

#[test]
fn test_bodo_config_load() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Create a temporary config file
    let mut temp_file = NamedTempFile::new().unwrap();
    let config_content = r#"
        root_script: "scripts/main.yaml"
        scripts_dirs: ["scripts/"]
        default_task:
          command: echo "Default Task"
        tasks:
          test:
            command: echo "Test Task"
    "#;
    write!(temp_file, "{}", config_content).unwrap();

    let config = BodoConfig::load(Some(temp_file.path().to_str().unwrap().to_string())).unwrap();
    assert_eq!(config.root_script, Some("scripts/main.yaml".to_string()));
    assert_eq!(config.scripts_dirs, Some(vec!["scripts/".to_string()]));
    assert!(config.default_task.is_some());
    assert_eq!(config.tasks.len(), 1);
    assert!(config.tasks.contains_key("test"));
}

#[test]
fn test_bodo_config_load_invalid_file() {
    let result = BodoConfig::load(Some("nonexistent_config.yaml".to_string()));
    assert!(result.is_err(), "Expected error loading nonexistent file");
}

#[test]
fn test_bodo_config_load_invalid_yaml() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut temp_file = NamedTempFile::new().unwrap();
    let invalid_yaml = r#"
        root_script: "scripts/main.yaml
        scripts_dirs: ["scripts/"]
    "#; // Missing closing quote

    write!(temp_file, "{}", invalid_yaml).unwrap();

    let result = BodoConfig::load(Some(temp_file.path().to_str().unwrap().to_string()));
    assert!(result.is_err(), "Expected error loading invalid YAML");
}

#[test]
fn test_validate_dependency_task() {
    let dep = Dependency::Task {
        task: "some_task".to_string(),
    };
    // Dependency is an untagged enum, no validation logic, but we can test serialization/deserialization
    let serialized = serde_yaml::to_string(&dep).unwrap();
    assert_eq!(serialized.trim(), "task: some_task");
}

#[test]
fn test_validate_dependency_command() {
    let dep = Dependency::Command {
        command: "echo 'Hello'".to_string(),
    };
    let serialized = serde_yaml::to_string(&dep).unwrap();
    assert_eq!(serialized.trim(), "command: echo 'Hello'");
}

#[test]
fn test_generate_schema() {
    let schema = BodoConfig::generate_schema();
    assert!(!schema.is_empty(), "Schema should not be empty");
    assert!(
        schema.contains("\"title\": \"BodoConfig\""),
        "Schema should contain BodoConfig title"
    );
}
