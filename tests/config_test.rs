use bodo::config::{
    BodoConfig, ConcurrentlyOptions, Dependency, TaskArgument, TaskConfig, WatchConfig,
};
use validator::Validate;

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
fn test_concurrently_options_validation() {
    let mut valid_opts = ConcurrentlyOptions {
        max_concurrent_tasks: Some(5),
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
fn test_watch_config_validation() {
    let valid_watch = WatchConfig {
        patterns: vec!["**/*.rs".into()],
        debounce_ms: 1000,
        ..Default::default()
    };
    assert!(valid_watch.validate().is_ok());

    let invalid_watch = WatchConfig {
        patterns: vec![],
        ..Default::default()
    };
    assert!(invalid_watch.validate().is_err());
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
