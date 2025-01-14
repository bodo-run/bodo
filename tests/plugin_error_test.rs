use bodo::{
    graph::{Graph, NodeKind, TaskData},
    plugin::{Plugin, PluginConfig},
    plugins::{path_plugin::PathPlugin, prefix_plugin::PrefixPlugin},
};

/// Test plugin initialization errors
#[tokio::test]
async fn test_plugin_init_errors() {
    let mut prefix_plugin = PrefixPlugin::new();

    // Test with invalid JSON type for prefix_format
    let config = PluginConfig {
        options: serde_json::json!({
            "prefix_format": 123 // should be a string
        })
        .as_object()
        .cloned(),
    };

    // Plugin should handle this gracefully and keep default format
    let result = prefix_plugin.on_init(&config).await;
    assert!(result.is_ok());
    assert_eq!(prefix_plugin.prefix_format, "[{}]"); // default format
}

/// Test path plugin with invalid paths
#[tokio::test]
async fn test_path_plugin_invalid_paths() {
    let mut path_plugin = PathPlugin::new();

    // Test with invalid path types in default_paths
    let config = PluginConfig {
        options: serde_json::json!({
            "default_paths": [123, true, null] // should be strings
        })
        .as_object()
        .cloned(),
    };

    // Plugin should handle invalid paths gracefully
    let result = path_plugin.on_init(&config).await;
    assert!(result.is_ok());
    assert!(path_plugin.default_paths.is_empty());
}

/// Test error handling during graph building
#[tokio::test]
async fn test_graph_build_errors() {
    let mut graph = Graph::new();

    // Add a task with invalid metadata
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "test".to_string(),
        description: None,
        command: None,
        working_dir: None,
    }));

    // Add some invalid metadata
    graph.nodes[task_id as usize]
        .metadata
        .insert("prefix".to_string(), "".to_string()); // empty prefix

    let mut prefix_plugin = PrefixPlugin::new();

    // Plugin should handle invalid metadata gracefully
    let result = prefix_plugin.on_graph_build(&mut graph).await;
    assert!(result.is_ok());

    // The prefix should be updated despite pre-existing invalid value
    let metadata = &graph.nodes[task_id as usize].metadata;
    assert_eq!(metadata.get("prefix"), Some(&"[test]".to_string()));
}

/// Test error recovery mechanisms
#[tokio::test]
async fn test_error_recovery() {
    let mut path_plugin = PathPlugin::new();
    let mut graph = Graph::new();

    // Add a task with an invalid exec_paths JSON
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "test".to_string(),
        description: None,
        command: None,
        working_dir: Some("/tmp".to_string()),
    }));

    // Add invalid JSON for exec_paths
    graph.nodes[task_id as usize]
        .metadata
        .insert("exec_paths".to_string(), "{invalid json}".to_string());

    // Plugin should recover and still set PATH with working_dir
    let result = path_plugin.on_graph_build(&mut graph).await;
    assert!(result.is_ok());

    let metadata = &graph.nodes[task_id as usize].metadata;
    let path = metadata.get("env.PATH").expect("PATH should be set");

    // Even though exec_paths was invalid, working_dir should be in PATH
    assert!(path.contains("/tmp"));
}

/// Test error propagation with multiple plugins
#[tokio::test]
async fn test_error_propagation_chain() {
    let mut graph = Graph::new();
    let mut prefix_plugin = PrefixPlugin::new();
    let mut path_plugin = PathPlugin::new();

    // Add a task that will cause issues
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "".to_string(), // empty name (might affect prefix plugin)
        description: None,
        command: None,
        working_dir: None,
    }));

    // Add invalid exec_paths (will affect path plugin)
    graph.nodes[task_id as usize]
        .metadata
        .insert("exec_paths".to_string(), "invalid".to_string());

    // Both plugins should handle their respective issues gracefully
    let prefix_result = prefix_plugin.on_graph_build(&mut graph).await;
    let path_result = path_plugin.on_graph_build(&mut graph).await;

    assert!(prefix_result.is_ok());
    assert!(path_result.is_ok());

    // Check that both plugins did their best despite the issues
    let metadata = &graph.nodes[task_id as usize].metadata;
    assert!(metadata.contains_key("prefix"));
    assert!(metadata.contains_key("env.PATH"));
}
