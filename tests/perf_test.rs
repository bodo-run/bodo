use std::{collections::HashMap, time::Instant};

use bodo::{
    graph::{Graph, NodeKind, TaskData},
    Result,
};

#[test]
fn test_add_many_nodes() -> Result<()> {
    let mut graph = Graph::new();
    let num_nodes = 1000;

    for i in 0..num_nodes {
        graph.add_node(NodeKind::Task(TaskData {
            name: format!("task_{}", i),
            description: Some("Test task".to_string()),
            command: Some("echo test".to_string()),
            working_dir: None,
            is_default: false,
            script_id: "test_script".to_string(),
            script_display_name: "Test".to_string(),
            env: HashMap::new(),
        }));
    }

    assert_eq!(graph.nodes.len(), num_nodes);
    Ok(())
}

#[test]
fn test_add_many_nodes_with_edges() -> Result<()> {
    let mut graph = Graph::new();
    let num_nodes = 1000;

    // Add nodes
    for i in 0..num_nodes {
        let node = graph.add_node(NodeKind::Task(TaskData {
            name: format!("task_{}", i),
            description: Some("Test task".to_string()),
            command: Some("echo test".to_string()),
            working_dir: None,
            is_default: false,
            script_id: "test_script".to_string(),
            script_display_name: "Test".to_string(),
            env: HashMap::new(),
        }));
        assert_eq!(node as usize, i);
    }

    // Add edges between consecutive nodes
    for i in 0..(num_nodes - 1) {
        let _ = graph.add_edge(i as u64, (i + 1) as u64);
    }

    assert_eq!(graph.nodes.len(), num_nodes);
    assert_eq!(graph.edges.len(), num_nodes - 1);
    Ok(())
}

#[test]
fn test_cycle_detection_performance() -> Result<()> {
    let mut graph = Graph::new();
    let num_nodes = 1000;

    // Add nodes
    for i in 0..num_nodes {
        graph.add_node(NodeKind::Task(TaskData {
            name: format!("task_{}", i),
            description: Some("Test task".to_string()),
            command: Some("echo test".to_string()),
            working_dir: None,
            is_default: false,
            script_id: "test_script".to_string(),
            script_display_name: "Test".to_string(),
            env: HashMap::new(),
        }));
    }

    // Add edges to create a long chain
    for i in 0..(num_nodes - 1) {
        let _ = graph.add_edge(i as u64, (i + 1) as u64);
    }

    assert!(!graph.has_cycle());

    // Add one edge to create a cycle
    let _ = graph.add_edge((num_nodes - 1) as u64, 0);
    assert!(graph.has_cycle());

    Ok(())
}
