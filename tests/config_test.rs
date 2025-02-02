use bodo::config::{Config, TaskConfig, TaskDependency, TaskType, WatchSettings};
use bodo::errors::{ConfigError, ValidationError};
use std::collections::HashMap;

#[test]
fn test_load_config_with_watch_settings() {
    let config = r#"
    tasks:
      build:
        command: cargo build
    
    watch:
      paths: ["src/**/*.rs"]
      exclude: ["target/"]
      interval: 500
    "#;

    let result = Config::from_yaml(config);
    assert!(result.is_ok());
}

#[test]
fn test_timeout_validation_edge_cases() {
    let mut valid_task = TaskConfig {
        task_type: TaskType::Command {
            command: "sleep 1".to_string(),
            args: None,
            env: None,
            cwd: None,
        },
        description: None,
        timeout: Some(1), // Minimum valid timeout instead of 0
        retries: None,
        retry_delay: None,
        dependencies: None,
        ignore_errors: None,
        concurrent: None,
        watch: None,
    };

    assert!(valid_task.validate().is_ok());

    let mut invalid_task = valid_task.clone();
    invalid_task.timeout = Some(0);
    assert_eq!(
        invalid_task.validate(),
        Err(ValidationError::InvalidTimeout)
    );
}

// Other existing tests below remain unchanged
