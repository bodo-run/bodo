use bodo::graph::TaskData;
use bodo::graph::{Edge, Graph, NodeKind};
use std::collections::HashMap;

#[test]
fn test_detect_and_format_cycle() {
    let mut graph = Graph::new();
    // Create three task nodes: A, B, C.
    let a = graph.add_node(NodeKind::Task(TaskData {
        name: "A".to_string(),
        description: None,
        command: Some("echo A".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "test".to_string(),
        script_display_name: "test".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    }));
    let b = graph.add_node(NodeKind::Task(TaskData {
        name: "B".to_string(),
        description: None,
        command: Some("echo B".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "test".to_string(),
        script_display_name: "test".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    }));
    let c = graph.add_node(NodeKind::Task(TaskData {
        name: "C".to_string(),
        description: None,
        command: Some("echo C".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "test".to_string(),
        script_display_name: "test".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    }));
    // Add edges to form a cycle: A->B, B->C, C->A.
    graph.edges.push(Edge { from: a, to: b });
    graph.edges.push(Edge { from: b, to: c });
    graph.edges.push(Edge { from: c, to: a });

    let cycle = graph.detect_cycle();
    assert!(cycle.is_some(), "Cycle should be detected");
    let cycle_nodes = cycle.unwrap();
    let error_msg = graph.format_cycle_error(&cycle_nodes);
    assert!(
        error_msg.contains("depends on"),
        "Error message should mention dependencies"
    );
}
