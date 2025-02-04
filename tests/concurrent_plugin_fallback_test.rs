use bodo::graph::{Graph, NodeKind, TaskData};
use bodo::plugin::Plugin;
use bodo::plugins::concurrent_plugin::ConcurrentPlugin;
use std::collections::HashMap; // ADDED IMPORT

#[test]
fn test_concurrent_plugin_fallback() {
    let mut plugin = ConcurrentPlugin::new();
    let mut graph = Graph::new();

    // Create a main task with no command so that concurrently runs are processed
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

    let main_node = &mut graph.nodes[main_task_id as usize];
    // Set up the main_task to have concurrent tasks
    main_node.metadata.insert(
        "concurrently".to_string(),
        "[\"nonexistent_task\"]".to_string(),
    );
    // Add a fallback task in task_registry with a key that ends with " test_task"
    let fallback_task = TaskData {
        name: "fallback".to_string(),
        description: None,
        command: Some("echo fallback".to_string()),
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
    let fallback_id = graph.add_node(NodeKind::Task(fallback_task));
    // Insert with key such that key.ends_with(" test_task") returns true.
    graph
        .task_registry
        .insert("prefix test_task".to_string(), fallback_id);

    let result = plugin.on_graph_build(&mut graph);
    assert!(
        result.is_ok(),
        "Concurrent fallback failed: {:?}",
        result.err()
    );

    // Check that a ConcurrentGroup node has been added and that it includes fallback_id.
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
    assert!(
        group_data.child_nodes.contains(&fallback_id),
        "Fallback task was not added in the concurrent group"
    );
}
