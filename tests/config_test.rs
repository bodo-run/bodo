use bodo::config::{
    BodoConfig, ConcurrentlyOptions, Dependency, TaskArgument, TaskConfig, WatchConfig,
};
use validator::Validate;

use validator::ValidationErrors;

#[test]
fn test_validate_task_name_reserved() {
    let mut config = TaskConfig::default();
    config._name_check = Some("default_task".to_string());
    let result = config.validate();
    assert!(matches!(result, Err(ValidationErrors { .. })));
}

#[test]
fn test_task_argument_boundaries() {
    // Test valid name length
    let valid_arg = TaskArgument {
        name: "a".repeat(64),
        description: Some("x".repeat(128)),
        ..Default::default()
    };
    assert!(valid_arg.validate().is_ok());

    // Test invalid name length
    let invalid_name = TaskArgument {
        name: "a".repeat(65),
        ..valid_arg.clone()
    };
    assert!(invalid_name.validate().is_err());

    // Test invalid description length
    let invalid_desc = TaskArgument {
        description: Some("x".repeat(129)),
        ..valid_arg
    };
    assert!(invalid_desc.validate().is_err());
}

#[test]
fn test_valid_task_config_variations() {
    // Valid task with command only
    let valid_with_cmd = TaskConfig {
        command: Some("echo valid".to_string()),
        ..Default::default()
    };
    assert!(valid_with_cmd.validate().is_ok());

    // Valid task with dependencies only
    let valid_with_deps = TaskConfig {
        pre_deps: vec![Dependency::Command {
            command: "echo pre".to_string(),
        }],
        ..Default::default()
    };
    assert!(valid_with_deps.validate().is_ok());

    // Valid task with concurrent deps
    let valid_concurrent = TaskConfig {
        concurrently: vec![Dependency::Task {
            task: "other-task".to_string(),
        }],
        ..Default::default()
    };
    assert!(valid_concurrent.validate().is_ok());
}

#[test]
fn test_watch_config_validation() {
    let valid_watch = WatchConfig {
        patterns: vec!["**/*.rs".to_string()],
        debounce_ms: 1000,
        ignore_patterns: vec!["target/**".to_string()],
        ..Default::default()
    };
    assert!(valid_watch.validate().is_ok());
}

#[test]
fn test_concurrently_options_validation() {
    let valid_opts = ConcurrentlyOptions {
        max_concurrent_tasks: Some(10),
        ..Default::default()
    };
    assert!(valid_opts.validate().is_ok());

    let invalid_opts = ConcurrentlyOptions {
        max_concurrent_tasks: Some(0),
        ..Default::default()
    };
    assert!(invalid_opts.validate().is_err());
}

#[test]
fn test_dependency_deserialization_variants() {
    let yaml = r#"
    - task: "complex::task::name"
    - command: "echo 'hello world' | grep hello"
    "#;
    let deps: Vec<Dependency> = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(deps.len(), 2);
    assert!(matches!(&deps[0], Dependency::Task { task } if task == "complex::task::name"));
    assert!(
        matches!(&deps[1], Dependency::Command { command } if command == "echo 'hello world' | grep hello")
    );
}

#[test]
fn test_timeout_validation_edge_cases() {
    let mut valid_task = TaskConfig {
        timeout: Some("1m30s".to_string()),
        ..Default::default()
    };
    valid_task._name_check = Some("valid".to_string());
    assert!(valid_task.validate().is_ok());

    valid_task.timeout = Some("10x".to_string());
    let result = valid_task.validate();
    assert!(matches!(result, Err(ValidationErrors { .. })));
}

#[test]
fn test_load_config_with_watch_settings() {
    let yaml = r#"
    scripts_dirs: ["scripts"]
    tasks:
      watch-task:
        watch:
          patterns: ["src/**/*"]
          debounce_ms: 200
    "#;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(temp_file.path(), yaml).unwrap();
    let result = BodoConfig::load(Some(temp_file.path().to_str().unwrap().to_string()));
    assert!(result.is_ok());
}

#[test]
fn test_no_op_task_validation() {
    let invalid_task = TaskConfig::default();
    let result = invalid_task.validate();
    assert!(matches!(result, Err(ValidationErrors { .. })));
}

#[test]
fn test_task_config_with_mixed_dependencies() {
    let valid_mixed = TaskConfig {
        command: Some("echo main".to_string()),
        pre_deps: vec![Dependency::Command {
            command: "echo pre".to_string(),
        }],
        post_deps: vec![Dependency::Task {
            task: "cleanup".to_string(),
        }],
        ..Default::default()
    };
    assert!(valid_mixed.validate().is_ok());
}

#[test]
fn test_task_name_validation() {
    let mut valid_task = TaskConfig {
        command: Some("echo test".into()),
        ..Default::default()
    };

    // Test valid names
    valid_task._name_check = Some("valid-task".into());
    assert!(valid_task.validate().is_ok());

    // Test reserved names
    for name in ["watch", "default_task", "pre_deps"] {
        valid_task._name_check = Some(name.into());
        let result = valid_task.validate();
        assert!(result.is_err(), "Should reject reserved name: {}", name);
    }

    // Test invalid characters
    for name in ["test/name", "..test", "test.name"] {
        valid_task._name_check = Some(name.into());
        let result = valid_task.validate();
        assert!(result.is_err(), "Should reject invalid chars in: {}", name);
    }

    // Test length boundaries
    let mut long_name = String::with_capacity(101);
    long_name.extend(std::iter::repeat('a').take(101));
    valid_task._name_check = Some(long_name);
    assert!(valid_task.validate().is_err());
}

#[test]
fn test_task_config_validation() {
    // Valid config with command
    let valid_with_command = TaskConfig {
        command: Some("echo valid".into()),
        ..Default::default()
    };
    assert!(valid_with_command.validate().is_ok());

    // Valid config with dependencies
    let valid_with_deps = TaskConfig {
        pre_deps: vec![Dependency::Command {
            command: "echo pre".into(),
        }],
        ..Default::default()
    };
    assert!(valid_with_deps.validate().is_ok());

    // Invalid empty config
    let invalid_empty = TaskConfig::default();
    assert!(invalid_empty.validate().is_err());
}

#[test]
fn test_timeout_validation() {
    let mut valid_task = TaskConfig {
        command: Some("echo test".into()),
        timeout: Some("30s".into()),
        ..Default::default()
    };
    assert!(valid_task.validate().is_ok());

    valid_task.timeout = Some("invalid".into());
    assert!(valid_task.validate().is_err());
}

#[test]
fn test_task_argument_validation() {
    let valid_arg = TaskArgument {
        name: "arg1".into(),
        description: Some("Test argument".into()),
        required: false,
        default: Some("default".into()),
    };
    assert!(valid_arg.validate().is_ok());

    let invalid_name = TaskArgument {
        name: "a".repeat(65),
        ..valid_arg.clone()
    };
    assert!(invalid_name.validate().is_err());
}

#[test]
fn test_dependency_parsing() {
    let yaml = r#"
    - task: test-task
    - command: echo hello
    "#;

    let deps: Vec<Dependency> = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(deps.len(), 2);
    assert!(matches!(deps[0], Dependency::Task { .. }));
    assert!(matches!(deps[1], Dependency::Command { .. }));
}

#[test]
fn test_config_load_validation() {
    let yaml = r#"
tasks:
  invalid/name:
    command: echo should fail
    "#;

    let result = BodoConfig::load(Some(yaml.into()));
    assert!(result.is_err());
}
