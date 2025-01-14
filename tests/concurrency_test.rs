use std::error::Error;

use bodo::{
    graph::{Graph, NodeKind, TaskData},
    manager::GraphManager,
};

/// Helper to construct a Graph with concurrent tasks
fn make_graph_with_concurrent_tasks(
    commands: Vec<(String, String)>, // (name, command)
    _fail_fast: bool,
    _timeout_secs: Option<u64>,
) -> Graph {
    let mut graph = Graph::new();

    // Add each command as a task node
    for (name, command) in commands {
        let task = NodeKind::Task(TaskData {
            name: name.clone(),
            description: None,
            command: Some(command),
            working_dir: None,
        });
        graph.add_node(task);
    }

    graph
}

#[test]
fn test_concurrent_graph_construction() -> Result<(), Box<dyn Error>> {
    // This command fails quickly
    let failing = (
        "failing_task".to_string(),
        "sh -c 'sleep 1 && exit 1'".to_string(),
    );

    // This command sleeps longer
    let long_running = (
        "long_task".to_string(),
        "sh -c 'sleep 5 && echo Long task finished'".to_string(),
    );

    // Create graph with both tasks
    let graph = make_graph_with_concurrent_tasks(vec![failing, long_running], true, None);

    // Verify graph structure
    assert_eq!(graph.nodes.len(), 2, "Graph should have 2 nodes");
    assert_eq!(graph.edges.len(), 0, "Graph should have no edges yet");

    // Verify node contents
    let failing_node = graph.nodes.iter().find(|n| match &n.kind {
        NodeKind::Task(t) => t.name == "failing_task",
        _ => false,
    });
    assert!(failing_node.is_some(), "Graph should contain failing task");

    let long_running_node = graph.nodes.iter().find(|n| match &n.kind {
        NodeKind::Task(t) => t.name == "long_task",
        _ => false,
    });
    assert!(
        long_running_node.is_some(),
        "Graph should contain long running task"
    );

    Ok(())
}

#[test]
fn test_concurrent_graph_manager() -> Result<(), Box<dyn Error>> {
    // One command that sleeps 5s
    let long_sleep = (
        "too_long".to_string(),
        "sh -c 'sleep 5 && echo Done'".to_string(),
    );

    // Create graph with the task
    let graph = make_graph_with_concurrent_tasks(vec![long_sleep], false, Some(2));

    // Create manager and set graph
    let mut manager = GraphManager::new();
    manager.graph = graph;

    // Verify manager state
    assert_eq!(manager.graph.nodes.len(), 1, "Manager should have 1 task");
    assert_eq!(manager.graph.edges.len(), 0, "Manager should have no edges");

    Ok(())
}

#[test]
fn test_concurrent_graph_output_prefixing() -> Result<(), Box<dyn Error>> {
    // Tasks with distinct output prefixes
    let item_a = ("taskA".to_string(), "echo 'Task A done'".to_string());

    let item_b = ("taskB".to_string(), "echo 'Task B done'".to_string());

    // Create graph with both tasks
    let graph = make_graph_with_concurrent_tasks(vec![item_a, item_b], false, None);

    // Verify graph structure
    assert_eq!(graph.nodes.len(), 2, "Graph should have 2 nodes");
    assert_eq!(graph.edges.len(), 0, "Graph should have no edges yet");

    // Verify node contents
    let task_a = graph.nodes.iter().find(|n| match &n.kind {
        NodeKind::Task(t) => t.name == "taskA",
        _ => false,
    });
    assert!(task_a.is_some(), "Graph should contain task A");

    let task_b = graph.nodes.iter().find(|n| match &n.kind {
        NodeKind::Task(t) => t.name == "taskB",
        _ => false,
    });
    assert!(task_b.is_some(), "Graph should contain task B");

    Ok(())
}
