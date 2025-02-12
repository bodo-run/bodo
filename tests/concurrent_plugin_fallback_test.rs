use bodo::{
    graph::{Graph, NodeKind, TaskData},
    plugin::Plugin,
    plugins::concurrent_plugin::ConcurrentPlugin,
};
use std::collections::HashMap;

#[test]
fn test_concurrent_plugin_fallback() {
    let mut plugin = ConcurrentPlugin::new();
    let mut graph = Graph::new();

    // Create a main task
    let task_data_main = TaskData {
        name: "main_task".to_string(),
        description: None,
        command: None, // No command, will have concurrent tasks
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
        is_default: true,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
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
        concurrently: vec![],
        concurrently_options: Default::default(),
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
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
        concurrently: vec![],
        concurrently_options: Default::default(),
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
    };

    let child2_id = graph.add_node(NodeKind::Task(task_data_child2));

    // Register child tasks in task_registry
    graph
        .task_registry
        .insert("child_task1".to_string(), child1_id);
    graph
        .task_registry
        .insert("child_task2".to_string(), child2_id);

    // Set up the main_task to have a nonexistent concurrent task.
    // This should trigger the fallback search.
    let main_node = &mut graph.nodes[main_task_id as usize];
    main_node.metadata.insert(
        "concurrently".to_string(),
        "[\"nonexistent_task\"]".to_string(),
    );
    // Insert fallback in task_registry with a key that ends with " nonexistent_task".
    graph
        .task_registry
        .insert("prefix nonexistent_task".to_string(), child1_id);

    let result = plugin.on_graph_build(&mut graph);
    assert!(
        result.is_ok(),
        "Concurrent fallback failed: {:?}",
        result.err()
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
    let (_group_id, group_data) = &group_nodes[0];
    assert!(group_data.child_nodes.contains(&child1_id));
}
