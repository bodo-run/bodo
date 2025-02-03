// tests/graph_test.rs

use bodo::graph::{Graph, NodeKind, TaskData};
use std::collections::HashMap;

#[test]
fn test_graph_add_nodes_and_edges() {
    let mut graph = Graph::new();

    let task_data1 = TaskData {
        name: "task1".to_string(),
        description: None,
        command: Some("echo task1".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    };
    let node_id1 = graph.add_node(NodeKind::Task(task_data1));

    let task_data2 = TaskData {
        name: "task2".to_string(),
        description: None,
        command: Some("echo task2".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    };
    let node_id2 = graph.add_node(NodeKind::Task(task_data2));

    graph.add_edge(node_id1, node_id2).unwrap();

    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 1);
    assert_eq!(graph.edges[0].from, node_id1);
    assert_eq!(graph.edges[0].to, node_id2);
}

#[test]
fn test_graph_detect_cycle() {
    let mut graph = Graph::new();

    let node_id1 = graph.add_node(NodeKind::Command(bodo::graph::CommandData {
        raw_command: "echo command1".to_string(),
        description: None,
        working_dir: None,
        env: HashMap::new(),
        watch: None,
    }));

    let node_id2 = graph.add_node(NodeKind::Command(bodo::graph::CommandData {
        raw_command: "echo command2".to_string(),
        description: None,
        working_dir: None,
        env: HashMap::new(),
        watch: None,
    }));

    let node_id3 = graph.add_node(NodeKind::Command(bodo::graph::CommandData {
        raw_command: "echo command3".to_string(),
        description: None,
        working_dir: None,
        env: HashMap::new(),
        watch: None,
    }));

    graph.add_edge(node_id1, node_id2).unwrap();
    graph.add_edge(node_id2, node_id3).unwrap();
    graph.add_edge(node_id3, node_id1).unwrap(); // This creates a cycle

    let cycle = graph.detect_cycle();
    assert!(cycle.is_some());
}

#[test]
fn test_graph_topological_sort() {
    let mut graph = Graph::new();

    let node_id1 = graph.add_node(NodeKind::Command(bodo::graph::CommandData {
        raw_command: "echo command1".to_string(),
        description: None,
        working_dir: None,
        env: HashMap::new(),
        watch: None,
    }));

    let node_id2 = graph.add_node(NodeKind::Command(bodo::graph::CommandData {
        raw_command: "echo command2".to_string(),
        description: None,
        working_dir: None,
        env: HashMap::new(),
        watch: None,
    }));

    graph.add_edge(node_id1, node_id2).unwrap();

    let sorted = graph.topological_sort().unwrap();
    assert_eq!(sorted.len(), 2);
    assert!(sorted[0] == node_id1 && sorted[1] == node_id2);
}
