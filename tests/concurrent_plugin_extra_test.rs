use bodo::graph::{Graph, NodeKind, TaskData};
use bodo::plugin::Plugin;
use bodo::plugins::concurrent_plugin::ConcurrentPlugin;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_concurrent_plugin_invalid_object_dependency() {
    let mut plugin = ConcurrentPlugin::new();
    let mut graph = Graph::new();

    let task_data_main = TaskData {
        name: "main_task".to_string(),
        description: None,
        command: None, // No command, will have concurrent tasks
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
        is_default: true,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    };

    let main_task_id = graph.add_node(NodeKind::Task(Box::new(task_data_main)));

    // Set up the main_task to have a nonexistent concurrent task
    let main_node = &mut graph.nodes[main_task_id as usize];
    main_node.metadata.insert(
        "concurrently".to_string(),
        "[\"nonexistent_task\"]".to_string(),
    );

    // Apply the plugin
    let result = plugin.on_graph_build(&mut graph);
    assert!(
        result.is_err(),
        "Expected error due to nonexistent task in object dependency"
    );
    let error = result.unwrap_err();
    assert!(
        matches!(error, bodo::errors::BodoError::PluginError(_)),
        "Expected PluginError, got {:?}",
        error
    );
}

#[test]
fn test_concurrent_plugin_empty_concurrently() {
    let mut plugin = ConcurrentPlugin::new();
    let mut graph = Graph::new();

    // Create a main task
    let task_data_main = TaskData {
        name: "main_task".to_string(),
        description: None,
        command: None,
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
        is_default: true,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    };

    let main_task_id = graph.add_node(NodeKind::Task(Box::new(task_data_main)));

    // Set up the main_task with empty 'concurrently' metadata
    let main_node = &mut graph.nodes[main_task_id as usize];
    main_node
        .metadata
        .insert("concurrently".to_string(), "[]".to_string());

    // Apply the plugin
    let result = plugin.on_graph_build(&mut graph);
    assert!(
        result.is_ok(),
        "Plugin on_graph_build returned an error: {:?}",
        result.unwrap_err()
    );

    // Check that a ConcurrentGroup node has been added with no children
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
    assert_eq!(
        group_data.child_nodes.len(),
        0,
        "Expected no child nodes in the group"
    );
}

#[test]
fn test_concurrent_plugin_nonexistent_task_in_object() {
    let mut plugin = ConcurrentPlugin::new();
    let mut graph = Graph::new();

    let task_data_main = TaskData {
        name: "main_task".to_string(),
        description: None,
        command: None,
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
        is_default: true,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    };
    let main_task_id = graph.add_node(NodeKind::Task(Box::new(task_data_main)));
    let main_node = &mut graph.nodes[main_task_id as usize];
    main_node.metadata.insert(
        "concurrently".to_string(),
        serde_json::to_string(&json!([{"task": "nonexistent"}])).unwrap(),
    );
    let result = plugin.on_graph_build(&mut graph);
    assert!(
        result.is_err(),
        "Expected error for nonexistent task in object dependency"
    );
}

#[test]
fn test_concurrent_plugin_invalid_dependency_format() {
    let mut plugin = ConcurrentPlugin::new();
    let mut graph = Graph::new();

    let task_data_main = TaskData {
        name: "main_task".to_string(),
        description: None,
        command: None,
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
        is_default: true,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    };

    let main_task_id = graph.add_node(NodeKind::Task(Box::new(task_data_main)));
    let main_node = &mut graph.nodes[main_task_id as usize];
    main_node.metadata.insert(
        "concurrently".to_string(),
        "[123, true]".to_string(), // Invalid format
    );

    let result = plugin.on_graph_build(&mut graph);
    assert!(
        result.is_err(),
        "Expected error due to invalid dependency format, but got success"
    );
    let error = result.unwrap_err();
    assert!(
        matches!(error, bodo::errors::BodoError::PluginError(_)),
        "Expected PluginError, got {:?}",
        error
    );
}
