use bodo::graph::{Graph, NodeKind, TaskData};
use bodo::plugin::Plugin;
use bodo::plugins::concurrent_plugin::ConcurrentPlugin;
use std::collections::HashMap;

#[test]
fn test_concurrent_plugin() {
    let mut plugin = ConcurrentPlugin::new();

    let mut graph = Graph::new();

    // Create tasks
    let task_data_main = TaskData {
        name: "main_task".to_string(),
        description: None,
        command: None, // No command, will have concurrent tasks
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: true,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    };

    let main_task_id = graph.add_node(NodeKind::Task(task_data_main));

    let task_data_child1 = TaskData {
        name: "child_task1".to_string(),
        description: None,
        command: Some("echo Child 1".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    };

    let child1_id = graph.add_node(NodeKind::Task(task_data_child1));

    let task_data_child2 = TaskData {
        name: "child_task2".to_string(),
        description: None,
        command: Some("echo Child 2".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    };

    let child2_id = graph.add_node(NodeKind::Task(task_data_child2));

    // Register child tasks in task_registry
    graph
        .task_registry
        .insert("child_task1".to_string(), child1_id);
    graph
        .task_registry
        .insert("child_task2".to_string(), child2_id);

    // Set up the main_task to have concurrent tasks
    let main_node = &mut graph.nodes[main_task_id as usize];
    // Set the metadata 'concurrently' directly as a JSON array string
    main_node.metadata.insert(
        "concurrently".to_string(),
        "[\"child_task1\", \"child_task2\"]".to_string(),
    );

    // Apply the plugin
    let result = plugin.on_graph_build(&mut graph);
    assert!(
        result.is_ok(),
        "Plugin on_graph_build returned an error: {:?}",
        result.unwrap_err()
    );

    // Check that a ConcurrentGroup node has been added
    let group_nodes: Vec<_> = graph
        .nodes
        .iter()
        .filter_map(|node| {
            if let NodeKind::ConcurrentGroup(group_data) = &node.kind {
                Some((node.id, group_data))
            } else {
                None
            }
        })
        .collect();

    assert_eq!(group_nodes.len(), 1, "Expected one concurrent group node");

    let (group_id, group_data) = &group_nodes[0];
    assert_eq!(group_data.child_nodes.len(), 2);
    assert!(group_data.child_nodes.contains(&child1_id));
    assert!(group_data.child_nodes.contains(&child2_id));

    // Check that edges have been added appropriately
    // Edge from main_task to group
    assert!(graph
        .edges
        .iter()
        .any(|edge| edge.from == main_task_id && edge.to == *group_id));

    // Edges from group to child tasks
    assert!(graph
        .edges
        .iter()
        .any(|edge| edge.from == *group_id && edge.to == child1_id));
    assert!(graph
        .edges
        .iter()
        .any(|edge| edge.from == *group_id && edge.to == child2_id));
}
