use std::{collections::HashMap, time::Instant};

use bodo::graph::{Graph, NodeKind, TaskData};

#[test]
fn test_graph_large_insertion() {
    let start = Instant::now();
    let mut g = Graph::new();

    for i in 0..10000 {
        g.add_node(NodeKind::Task(TaskData {
            name: format!("task_{}", i),
            description: Some("Test task".to_string()),
            command: Some("echo test".to_string()),
            working_dir: None,
            is_default: false,
            script_name: Some("Test".to_string()),
            env: HashMap::new(),
        }));
    }

    let duration = start.elapsed();
    assert!(duration.as_millis() < 1000, "Node insertion took too long");
}

#[test]
fn test_graph_large_edge_creation() {
    let mut g = Graph::new();

    // First create nodes
    for i in 0..10000 {
        let node = g.add_node(NodeKind::Task(TaskData {
            name: format!("task_{}", i),
            description: Some("Test task".to_string()),
            command: Some("echo test".to_string()),
            working_dir: None,
            is_default: false,
            script_name: Some("Test".to_string()),
            env: HashMap::new(),
        }));
        assert_eq!(node, i);
    }

    // Then time edge creation
    let start = Instant::now();
    for i in 0..9999 {
        g.add_edge(i, i + 1).unwrap();
    }

    let duration = start.elapsed();
    assert!(duration.as_millis() < 1000, "Edge creation took too long");
}

#[test]
fn test_graph_large_cycle_detection() {
    let mut g = Graph::new();

    // Create nodes
    for i in 0..10000 {
        g.add_node(NodeKind::Task(TaskData {
            name: format!("task_{}", i),
            description: Some("Test task".to_string()),
            command: Some("echo test".to_string()),
            working_dir: None,
            is_default: false,
            script_name: Some("Test".to_string()),
            env: HashMap::new(),
        }));
    }

    // Create edges in a chain (no cycle)
    for i in 0..9999 {
        g.add_edge(i, i + 1).unwrap();
    }

    let start = Instant::now();
    assert!(!g.has_cycle());
    let duration = start.elapsed();

    assert!(duration.as_millis() < 1000, "Cycle detection took too long");
}
