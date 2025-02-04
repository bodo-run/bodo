use bodo::config::{BodoConfig, Dependency, TaskConfig, WatchConfig};
use validator::Validate;
use validator::ValidationErrors;

#[test]
fn test_validate_task_config_no_command_no_deps() {
    let config = TaskConfig {
        command: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        ..Default::default()
    };
    let result = config.validate();
    assert!(result.is_err());
}

#[test]
fn test_validate_task_config_with_deps() {
    let config = TaskConfig {
        command: None,
        pre_deps: vec![Dependency::Task {
            task: "some_task".to_string(),
        }],
        ..Default::default()
    };
    let result = config.validate();
    assert!(result.is_ok());
}

#[test]
fn test_validate_watch_config_empty_patterns() {
    let config = WatchConfig {
        patterns: vec![],
        debounce_ms: 500,
        ignore_patterns: vec![],
        auto_watch: false,
    };
    let result = config.validate();
    assert!(result.is_err());
}
