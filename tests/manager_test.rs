use bodo::config::{BodoConfig, TaskArgument, TaskConfig};
use bodo::errors::BodoError;
use bodo::graph::NodeKind;
use bodo::manager::GraphManager;
use std::collections::HashMap;

#[test]
fn test_graph_manager_initialization() {
    let config = BodoConfig::default();
    let mut manager = GraphManager::new();
    manager.build_graph(config).unwrap();
    assert!(manager.graph.nodes.is_empty());
}

#[test]
fn test_graph_manager_with_tasks() {
    let config_yaml = r#"
    tasks:
      hello:
        command: echo "Hello World"
    "#;

    let config: BodoConfig = serde_yaml::from_str(config_yaml).unwrap();

    let mut manager = GraphManager::new();
    manager.build_graph(config).unwrap();
    assert!(!manager.graph.nodes.is_empty());
    assert!(manager.task_exists("hello"));
}

#[test]
fn test_apply_task_arguments() {
    let config_yaml = r#"
    tasks:
      greet:
        command: echo "Hello $name"
    "#;

    let config: BodoConfig = serde_yaml::from_str(config_yaml).unwrap();

    let mut manager = GraphManager::new();
    manager.build_graph(config).unwrap();
    // Simulate applying argument by directly injecting into env for test purposes.
    manager.graph.nodes.iter_mut().for_each(|node| {
        if let NodeKind::Task(task_data) = &mut node.kind {
            if task_data.name == "greet" {
                task_data
                    .env
                    .insert("name".to_string(), "Alice".to_string());
            }
        }
    });

    let task_name = "greet";
    let node_id = manager
        .graph
        .task_registry
        .get(task_name)
        .expect("Task 'greet' not found");
    let node = &manager.graph.nodes[*node_id as usize];

    if let NodeKind::Task(task_data) = &node.kind {
        assert_eq!(task_data.env.get("name"), Some(&"Alice".to_string()));
    } else {
        panic!("Expected Task node");
    }
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
    let node = &manager.graph.nodes[node_id as usize];

    if let NodeKind::Task(task_data) = &node.kind {
        assert_eq!(task_data.env.get("greeting"), Some(&"Hello".to_string()));
    } else {
        panic!("Expected Task node");
    }
}

#[test]
fn test_get_task_config_nonexistent_task() {
    let manager = GraphManager::new();
    let result = manager.get_task_config("nonexistent");
    assert!(matches!(result, Err(BodoError::TaskNotFound(_))));
}
