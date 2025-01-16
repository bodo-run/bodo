use bodo::graph::{Graph, NodeKind, TaskData};
use bodo::GraphManager;
use std::time::Instant;

#[test]
fn test_graph_large_insertion() {
    let mut g = Graph::new();
    let start = Instant::now();

    for i in 0..10_000 {
        g.add_node(NodeKind::Task(TaskData {
            name: format!("task_{i}"),
            description: None,
            command: Some(format!("echo task_{i}")),
            working_dir: None,
            is_default: false,
            script_name: Some("Test".to_string()),
        }));
    }

    let duration = start.elapsed();
    println!("Time taken for 10k insertions: {:?}", duration);
    assert_eq!(g.nodes.len(), 10_000);
    // Assuming reasonable performance on modern hardware
    assert!(
        duration.as_millis() < 1000,
        "Insertion took too long: {:?}",
        duration
    );
}

#[test]
fn test_graph_large_edge_creation() {
    let mut g = Graph::new();
    let mut node_ids = Vec::new();

    // Create nodes first
    for i in 0..1000 {
        let node = g.add_node(NodeKind::Task(TaskData {
            name: format!("task_{i}"),
            description: None,
            command: Some(format!("echo task_{i}")),
            working_dir: None,
            is_default: false,
            script_name: Some("Test".to_string()),
        }));
        node_ids.push(node);
    }

    let start = Instant::now();
    // Create a chain of dependencies: task_0 -> task_1 -> task_2 -> ...
    for i in 0..999 {
        g.add_edge(node_ids[i], node_ids[i + 1]).unwrap();
    }

    let duration = start.elapsed();
    println!("Time taken for 999 edge creations: {:?}", duration);
    assert!(
        duration.as_millis() < 500,
        "Edge creation took too long: {:?}",
        duration
    );
}

#[test]
fn test_large_graph_manager() {
    let mut manager = GraphManager::new();
    let start = Instant::now();

    // Build a large graph through the manager
    let mut g = Graph::new();
    for i in 0..1000 {
        g.add_node(NodeKind::Task(TaskData {
            name: format!("task_{i}"),
            description: None,
            command: Some(format!("echo task_{i}")),
            working_dir: None,
            is_default: false,
            script_name: Some("Test".to_string()),
        }));
    }
    manager.graph = g;

    let duration = start.elapsed();
    println!(
        "Time taken for loading 1k tasks through manager: {:?}",
        duration
    );
    assert!(
        duration.as_millis() < 500,
        "Graph manager operations took too long: {:?}",
        duration
    );
}
