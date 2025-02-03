// tests/graph_test.rs

use bodo::graph::{Graph, NodeKind, TaskData};
use std::collections::HashMap;

#[test]
fn test_add_node() {
    let mut graph = Graph::new();
    let node_id = graph.add_node(NodeKind::Task(TaskData {
        name: "test_task".to_string(),
        description: None,
        command: None,
        working_dir: None,
        env: Default::default(),
        exec_paths: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    }));
    assert_eq!(node_id, 0);
    assert_eq!(graph.nodes.len(), 1);
}

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
fn test_detect_cycle() {
    let mut graph = Graph::new();
    let node_id1 = graph.add_node(NodeKind::Task(TaskData {
        name: "task1".to_string(),
        description: None,
        command: None,
        working_dir: None,
        env: Default::default(),
        exec_paths: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    }));
    let node_id2 = graph.add_node(NodeKind::Task(TaskData {
        name: "task2".to_string(),
        description: None,
        command: None,
        working_dir: None,
        env: Default::default(),
        exec_paths: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    }));
    graph.add_edge(node_id1, node_id2).unwrap();
    graph.add_edge(node_id2, node_id1).unwrap();

    let cycle = graph.detect_cycle();
    assert!(cycle.is_some());
}

#[test]
fn test_topological_sort() {
    let mut graph = Graph::new();

    let node_a = graph.add_node(NodeKind::Task(TaskData {
        name: "A".to_string(),
        description: None,
        command: None,
        working_dir: None,
        env: Default::default(),
        exec_paths: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
    }));
    let node_b = graph.add_node(NodeKind::Task(TaskData {
        name: "B".to_string(),
        description: None,
        command: None,
        working_dir: None,
        env: Default::default(),
        exec_paths: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
    }));
    let node_c = graph.add_node(NodeKind::Task(TaskData {
        name: "C".to_string(),
        description: None,
        command: None,
        working_dir: None,
        env: Default::default(),
        exec_paths: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
    }));
    graph.add_edge(node_a, node_b).unwrap();
    graph.add_edge(node_b, node_c).unwrap();

    let sorted = graph.topological_sort().unwrap();
    assert_eq!(sorted.len(), 3);
    assert!(sorted[0] == node_a && sorted[1] == node_b && sorted[2] == node_c);
}

#[test]
fn test_add_invalid_edge() {
    let mut graph = Graph::new();
    let result = graph.add_edge(10, 20);
    assert!(result.is_err());
}

#[test]
fn test_format_cycle_error() {
    let mut graph = Graph::new();
    let node_id1 = graph.add_node(NodeKind::Task(TaskData {
        name: "task1".to_string(),
        description: None,
        command: None,
        working_dir: None,
        env: Default::default(),
        exec_paths: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    }));
    let node_id2 = graph.add_node(NodeKind::Task(TaskData {
        name: "task2".to_string(),
        description: None,
        command: None,
        working_dir: None,
        env: Default::default(),
        exec_paths: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    }));
    graph.add_edge(node_id1, node_id2).unwrap();
    graph.add_edge(node_id2, node_id1).unwrap();

    let cycle = graph.detect_cycle().unwrap();
    let error_message = graph.format_cycle_error(&cycle);
    assert!(error_message.contains("found cyclical dependency"));
}

#[test]
fn test_topological_sort_with_cycle() {
    let mut graph = Graph::new();
    let node_a = graph.add_node(NodeKind::Task(TaskData {
        name: "A".to_string(),
        description: None,
        command: None,
        working_dir: None,
        env: Default::default(),
        exec_paths: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
    }));
    let node_b = graph.add_node(NodeKind::Task(TaskData {
        name: "B".to_string(),
        description: None,
        command: None,
        working_dir: None,
        env: Default::default(),
        exec_paths: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
    }));
    graph.add_edge(node_a, node_b).unwrap();
    graph.add_edge(node_b, node_a).unwrap();

    let sorted = graph.topological_sort();

    assert!(sorted.is_err(), "Expected an error due to cycle in graph");
}
