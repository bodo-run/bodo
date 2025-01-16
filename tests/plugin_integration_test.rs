use bodo::{
    graph::{Graph, NodeKind, TaskData},
    plugin::{Plugin, PluginConfig},
    plugins::{path_plugin::PathPlugin, prefix_plugin::PrefixPlugin},
};

/// Test that multiple plugins can work together on the same graph
#[tokio::test]
async fn test_prefix_and_path_plugin_integration() {
    // Initialize plugins with custom configs
    let mut prefix_plugin = PrefixPlugin::new();
    let prefix_config = PluginConfig {
        options: serde_json::json!({
            "prefix_format": "[{}]"
        })
        .as_object()
        .cloned(),
    };
    prefix_plugin.on_init(&prefix_config).await.unwrap();

    let mut path_plugin = PathPlugin::new();
    let path_config = PluginConfig {
        options: serde_json::json!({
            "default_paths": ["/usr/local/bin"]
        })
        .as_object()
        .cloned(),
    };
    path_plugin.on_init(&path_config).await.unwrap();

    // Create a test graph
    let mut graph = Graph::new();
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "build".to_string(),
        description: None,
        command: Some("make build".to_string()),
        working_dir: Some("/tmp".to_string()),
        is_default: false,
        script_name: Some("Test".to_string()),
    }));

    // Add some metadata that path plugin will interpret
    graph.nodes[task_id as usize]
        .metadata
        .insert("exec_paths".to_string(), "[\"/opt/bin\"]".to_string());

    // Run both plugins on the graph
    prefix_plugin.on_graph_build(&mut graph).await.unwrap();
    path_plugin.on_graph_build(&mut graph).await.unwrap();

    // Verify both plugins' effects are present
    let node_metadata = &graph.nodes[task_id as usize].metadata;

    // Check prefix plugin's contribution
    assert_eq!(node_metadata.get("prefix"), Some(&"[build]".to_string()));

    // Check path plugin's contribution
    let path_env = node_metadata.get("env.PATH").expect("PATH should be set");
    assert!(path_env.contains("/usr/local/bin")); // from default_paths
    assert!(path_env.contains("/opt/bin")); // from exec_paths
    assert!(path_env.contains("/tmp")); // from working_dir
}

/// Test that plugins are executed in the correct order
#[tokio::test]
async fn test_plugin_execution_order() {
    // In this test, we verify that plugins are executed in a specific order
    // For example, path plugin should run before any execution plugin that needs PATH
    // For now, we just verify our two plugins work in any order

    let mut graph = Graph::new();
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "test".to_string(),
        description: None,
        command: Some("echo test".to_string()),
        working_dir: Some("/tmp".to_string()),
        is_default: false,
        script_name: Some("Test".to_string()),
    }));

    // Try both orders to ensure no dependency between these plugins
    {
        let mut prefix_plugin = PrefixPlugin::new();
        let mut path_plugin = PathPlugin::new();

        // Order 1: prefix -> path
        prefix_plugin.on_graph_build(&mut graph).await.unwrap();
        path_plugin.on_graph_build(&mut graph).await.unwrap();

        let metadata = &graph.nodes[task_id as usize].metadata;
        assert!(metadata.contains_key("prefix"));
        assert!(metadata.contains_key("env.PATH"));
    }

    // Reset graph
    graph.nodes[task_id as usize].metadata.clear();

    {
        let mut prefix_plugin = PrefixPlugin::new();
        let mut path_plugin = PathPlugin::new();

        // Order 2: path -> prefix
        path_plugin.on_graph_build(&mut graph).await.unwrap();
        prefix_plugin.on_graph_build(&mut graph).await.unwrap();

        let metadata = &graph.nodes[task_id as usize].metadata;
        assert!(metadata.contains_key("prefix"));
        assert!(metadata.contains_key("env.PATH"));
    }
}

/// Test error propagation between plugins
#[tokio::test]
async fn test_plugin_error_propagation() {
    let mut graph = Graph::new();

    // Add a task with invalid exec_paths that path plugin can't parse
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "test".to_string(),
        description: None,
        command: Some("echo test".to_string()),
        working_dir: None,
        is_default: false,
        script_name: Some("Test".to_string()),
    }));

    // Add invalid JSON that path plugin can't parse
    graph.nodes[task_id as usize]
        .metadata
        .insert("exec_paths".to_string(), "invalid json".to_string());

    let mut path_plugin = PathPlugin::new();

    // Path plugin should handle the invalid JSON gracefully
    let result = path_plugin.on_graph_build(&mut graph).await;
    assert!(
        result.is_ok(),
        "Path plugin should handle invalid exec_paths gracefully"
    );

    // The PATH should still be set, just without the invalid paths
    let metadata = &graph.nodes[task_id as usize].metadata;
    assert!(metadata.contains_key("env.PATH"));
}
