// tests/cli_test.rs

use bodo::cli::{get_task_name, Args};
use bodo::config::BodoConfig;
use bodo::errors::BodoError;
use bodo::manager::GraphManager;

#[test]
fn test_get_task_name_with_task_and_subtask() {
    let args = Args {
        list: false,
        watch: false,
        auto_watch: false,
        debug: false,
        task: Some("deploy".to_string()),
        subtask: Some("prod".to_string()),
        args: vec![],
    };

    let mut manager = GraphManager::new();
    let config = BodoConfig::default();
    manager.build_graph(config).unwrap();

    manager
        .graph
        .task_registry
        .insert("deploy prod".to_string(), 1);

    let task_name = get_task_name(&args, &manager).unwrap();
    assert_eq!(task_name, "deploy prod");
}

#[test]
fn test_get_task_name_with_task_only() {
    let args = Args {
        list: false,
        watch: false,
        auto_watch: false,
        debug: false,
        task: Some("build".to_string()),
        subtask: None,
        args: vec![],
    };

    let mut manager = GraphManager::new();
    let config = BodoConfig::default();
    manager.build_graph(config).unwrap();

    manager.graph.task_registry.insert("build".to_string(), 1);

    let task_name = get_task_name(&args, &manager).unwrap();
    assert_eq!(task_name, "build");
}

#[test]
fn test_get_task_name_default_task() {
    let args = Args {
        list: false,
        watch: false,
        auto_watch: false,
        debug: false,
        task: None,
        subtask: None,
        args: vec![],
    };

    let mut manager = GraphManager::new();

    let config_yaml = r#"
    tasks:
      default:
        command: echo "Default Task"
    "#;

    let config: BodoConfig = serde_yaml::from_str(config_yaml).unwrap();
    manager.build_graph(config).unwrap();

    let task_name = get_task_name(&args, &manager).unwrap();
    assert_eq!(task_name, "default");
}

#[test]
fn test_get_task_name_no_task_specified() {
    let args = Args {
        list: false,
        watch: false,
        auto_watch: false,
        debug: false,
        task: None,
        subtask: None,
        args: vec![],
    };

    let mut manager = GraphManager::new();
    let config = BodoConfig::default();
    manager.build_graph(config).unwrap();

    let result = get_task_name(&args, &manager);
    assert!(matches!(result, Err(BodoError::NoTaskSpecified)));
}

#[test]
fn test_get_task_name_task_not_found() {
    let args = Args {
        list: false,
        watch: false,
        auto_watch: false,
        debug: false,
        task: Some("unknown_task".to_string()),
        subtask: None,
        args: vec![],
    };

    let mut manager = GraphManager::new();
    let config = BodoConfig::default();
    manager.build_graph(config).unwrap();

    let result = get_task_name(&args, &manager);
    assert!(matches!(result, Err(BodoError::TaskNotFound(task)) if task == "unknown_task"));
}
