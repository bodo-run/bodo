use bodo::config::{BodoConfig, TaskArgument, TaskConfig};
use bodo::errors::BodoError;
use bodo::graph::NodeKind;
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
fn test_apply_task_arguments_with_arguments() {
    let mut manager = GraphManager::new();
    let task_config = TaskConfig {
        command: Some("echo $greeting".to_string()),
        arguments: vec![TaskArgument {
            name: "greeting".to_string(),
            description: None,
            required: true,
            default: None,
        }],
        ..Default::default()
    };
    let mut tasks = HashMap::new();
    tasks.insert("hello".to_string(), task_config);
    let config = BodoConfig {
        tasks,
        ..Default::default()
    };
    manager.build_graph(config).unwrap();
    let result = manager.apply_task_arguments("hello", &["World".to_string()]);
    assert!(result.is_ok());

    let node_id = manager
        .graph
        .task_registry
        .get("hello")
        .cloned()
        .expect("Task 'hello' not found");
    let node = manager
        .graph
        .nodes
        .get(node_id as usize)
        .expect("Node not found");

    if let NodeKind::Task(task_data) = &node.kind {
        assert_eq!(task_data.env.get("greeting"), Some(&"World".to_string()));
    } else {
        panic!("Expected Task node");
    }
}

#[test]
fn test_apply_task_arguments_missing_required() {
    let mut manager = GraphManager::new();
    let task_config = TaskConfig {
        command: Some("echo $greeting".to_string()),
        arguments: vec![TaskArgument {
            name: "greeting".to_string(),
            description: None,
            required: true,
            default: None,
        }],
        ..Default::default()
    };
    let mut tasks = HashMap::new();
    tasks.insert("hello".to_string(), task_config);
    let config = BodoConfig {
        tasks,
        ..Default::default()
    };
    manager.build_graph(config).unwrap();
    let result = manager.apply_task_arguments("hello", &[]);
    assert!(result.is_err());
}

#[test]
fn test_apply_task_arguments_with_defaults() {
    let mut manager = GraphManager::new();
    let task_config = TaskConfig {
        command: Some("echo $greeting".to_string()),
        arguments: vec![TaskArgument {
            name: "greeting".to_string(),
            description: None,
            required: false,
            default: Some("Hello".to_string()),
        }],
        ..Default::default()
    };
    let mut tasks = HashMap::new();
    tasks.insert("hello".to_string(), task_config);
    let config = BodoConfig {
        tasks,
        ..Default::default()
    };
    manager.build_graph(config).unwrap();
    let result = manager.apply_task_arguments("hello", &[]);
    assert!(result.is_ok());

    let node_id = manager
        .graph
        .task_registry
        .get("hello")
        .cloned()
        .expect("Task 'hello' not found");
    let node = manager
        .graph
        .nodes
        .get(node_id as usize)
        .expect("Node not found");

    if let NodeKind::Task(task_data) = &node.kind {
        assert_eq!(task_data.env.get("greeting"), Some(&"Hello".to_string()));
    } else {
        panic!("Expected Task node");
    }
}
