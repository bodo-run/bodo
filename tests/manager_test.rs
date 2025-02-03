use bodo::config::BodoConfig;
use bodo::errors::BodoError;
use bodo::manager::GraphManager;
use std::collections::HashMap;

#[test]
fn test_build_graph() {
    let mut manager = GraphManager::new();
    let config = BodoConfig {
        root_script: None,
        scripts_dirs: None,
        tasks: HashMap::new(),
        env: HashMap::new(),
        exec_paths: vec![],
    };
    let result = manager.build_graph(config);
    assert!(result.is_ok());
}

#[test]
fn test_run_plugins_with_no_plugins() {
    let mut manager = GraphManager::new();
    let config = BodoConfig {
        root_script: None,
        scripts_dirs: None,
        tasks: HashMap::new(),
        env: HashMap::new(),
        exec_paths: vec![],
    };
    manager.build_graph(config).unwrap();
    let result = manager.run_plugins(None);
    assert!(result.is_ok());
}

#[test]
fn test_get_task_config_nonexistent_task() {
    let manager = GraphManager::new();
    let result = manager.get_task_config("nonexistent");
    assert!(matches!(result, Err(BodoError::TaskNotFound(_))));
}

#[test]
fn test_apply_task_arguments_no_arguments() {
    let mut manager = GraphManager::new();
    let config = BodoConfig {
        root_script: None,
        scripts_dirs: None,
        tasks: HashMap::new(),
        env: HashMap::new(),
        exec_paths: vec![],
    };
    manager.build_graph(config).unwrap();
    let result = manager.apply_task_arguments("default", &[]);
    assert!(result.is_err()); // No task named "default"
}
