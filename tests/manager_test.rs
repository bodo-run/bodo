// tests/manager_test.rs

use bodo::{config::BodoConfig, manager::GraphManager};

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
        args:
          - name: name
            required: true
    "#;

    let config: BodoConfig = serde_yaml::from_str(config_yaml).unwrap();

    let mut manager = GraphManager::new();
    manager.build_graph(config).unwrap();
    let args = vec!["Alice".to_string()];
    manager.apply_task_arguments("greet", &args).unwrap();

    let node_id = manager.graph.task_registry.get("greet").unwrap();
    let node = &manager.graph.nodes[*node_id as usize];
    if let bodo::graph::NodeKind::Task(task_data) = &node.kind {
        assert_eq!(task_data.env.get("name"), Some(&"Alice".to_string()));
    } else {
        panic!("Expected Task node");
    }
}
