// tests/timeout_plugin_test.rs

use std::collections::HashMap;

use bodo::graph::{Graph, NodeKind, TaskData};
use bodo::plugin::Plugin;
use bodo::plugins::timeout_plugin::TimeoutPlugin;

#[test]
fn test_timeout_plugin() {
    let mut plugin = TimeoutPlugin::new();

    let config = bodo::plugin::PluginConfig {
        options: Some(
            serde_json::json!({
                "default_paths": ["/usr/local/bin"],
                "preserve_path": false
            })
            .as_object()
            .cloned()
            .unwrap(),
        ),
        ..Default::default()
    };

    plugin.on_init(&config).unwrap();

    let mut graph = Graph::new();

    let task_data = TaskData {
        name: "test_task".to_string(),
        description: None,
        command: Some("sleep 5".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    };

    let node_id = graph.add_node(NodeKind::Task(task_data));

    // Set up a node with a timeout in metadata
    let node = &mut graph.nodes[node_id as usize];
    node.metadata
        .insert("timeout".to_string(), "2s".to_string());

    // Apply the plugin
    plugin.on_graph_build(&mut graph).unwrap();

    // Check that the timeout_seconds metadata is set
    let node = &graph.nodes[node_id as usize];
    assert_eq!(node.metadata.get("timeout_seconds"), Some(&"2".to_string()));
}

#[test]
fn test_invalid_timeout_plugin() {
    let mut plugin = TimeoutPlugin::new();

    let mut graph = Graph::new();

    let task_data = TaskData {
        name: "test_task_invalid_timeout".to_string(),
        description: None,
        command: Some("sleep 5".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    };

    let node_id = graph.add_node(NodeKind::Task(task_data));

    // Set up a node with an invalid timeout in metadata
    let node = &mut graph.nodes[node_id as usize];
    node.metadata
        .insert("timeout".to_string(), "invalid_timeout".to_string());

    // Apply the plugin, should return an error
    let result = plugin.on_graph_build(&mut graph);

    assert!(result.is_err(), "Expected error due to invalid timeout");
}
