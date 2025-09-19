use bodo::graph::{NodeKind, TaskData};
use bodo::manager::GraphManager;
use std::collections::HashMap;

#[test]
fn test_task_exists_true() {
    let mut manager = GraphManager::new();
    let node_id = manager.graph.add_node(NodeKind::Task(Box::new(TaskData {
        name: "existing".to_string(),
        description: None,
        command: Some("echo exist".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    })));
    manager
        .graph
        .task_registry
        .insert("existing".to_string(), node_id);
    assert!(manager.task_exists("existing"));
}

#[test]
fn test_task_exists_false() {
    let manager = GraphManager::new();
    assert!(!manager.task_exists("nonexistent"));
}
