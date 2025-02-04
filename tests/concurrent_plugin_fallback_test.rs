use bodo::graph::{Graph, NodeKind, TaskData};
use bodo::plugins::concurrent_plugin::ConcurrentPlugin;
use bodo::Plugin;
use serde_json::json;
use std::collections::HashMap; // Added to bring the Plugin trait into scope

#[test]
fn test_concurrent_plugin_fallback_search() {
    let mut graph = Graph::new();
    // Create a main task with "concurrently" metadata set to a simple string "test_task"
    let main_task = TaskData {
        name: "main".to_string(),
        description: None,
        command: Some("echo main".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: true,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    };
    let main_id = graph.add_node(NodeKind::Task(main_task));
    // Set concurrently metadata to a JSON string representing "test_task"
    graph.nodes[main_id as usize]
        .metadata
        .insert("concurrently".to_string(), "\"test_task\"".to_string());

    // Add a fallback task with registry key "prefix test_task"
    let fallback_task = TaskData {
        name: "fallback".to_string(),
        description: None,
        command: Some("echo fallback".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "script2".to_string(),
        script_display_name: "script2".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    };
    let fallback_id = graph.add_node(NodeKind::Task(fallback_task));
    graph
        .task_registry
        .insert("prefix test_task".to_string(), fallback_id);

    let mut plugin = ConcurrentPlugin::new();
    let res = plugin.on_graph_build(&mut graph);
    assert!(res.is_ok());

    // Retrieve the concurrent group added by the plugin and check that fallback_id is included.
    let group_nodes: Vec<_> = graph
        .nodes
        .iter()
        .filter(|node| matches!(node.kind, NodeKind::ConcurrentGroup(_)))
        .collect();
    assert_eq!(group_nodes.len(), 1, "Expected one concurrent group node");
    if let NodeKind::ConcurrentGroup(ref group_data) = group_nodes[0].kind {
        assert!(group_data.child_nodes.contains(&fallback_id));
    } else {
        panic!("Expected a concurrent group node");
    }
}

#[test]
fn test_concurrent_plugin_invalid_dependency_format() {
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

    // Set up the main_task to have an invalid concurrent dependency
    let main_node = &mut graph.nodes[main_task_id as usize];
    main_node
        .metadata
        .insert("concurrently".to_string(), "[123, true]".to_string());

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
