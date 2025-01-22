use bodo::{
    errors::BodoError,
    graph::{Graph, NodeKind, TaskData},
    plugin::Plugin,
    plugins::watch_plugin::WatchPlugin,
    Result,
};
use serde_json::json;
use std::collections::HashMap;

#[tokio::test]
async fn test_watch_plugin_valid_config() -> Result<()> {
    let mut graph = Graph::new();
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "watch_test".to_string(),
        description: Some("Test watch functionality".to_string()),
        command: Some("echo 'File changed'".to_string()),
        working_dir: None,
        env: HashMap::new(),
        is_default: false,
        script_name: None,
    }));

    // Add valid watch configuration
    let node = &mut graph.nodes[task_id as usize];
    node.metadata.insert(
        "watch".to_string(),
        json!({
            "patterns": ["."],
            "debounce_ms": 500,
            "ignore_patterns": ["target/"]
        })
        .to_string(),
    );

    let mut plugin = WatchPlugin::new();
    plugin.on_graph_build(&mut graph).await?;

    // If we reach this point without errors, the test passes
    Ok(())
}

#[tokio::test]
async fn test_watch_plugin_invalid_config() {
    let mut graph = Graph::new();
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "invalid_watch_test".to_string(),
        description: Some("Test invalid config".to_string()),
        command: Some("echo 'Should fail'".to_string()),
        working_dir: None,
        env: HashMap::new(),
        is_default: false,
        script_name: None,
    }));

    // Add invalid JSON configuration
    let node = &mut graph.nodes[task_id as usize];
    node.metadata
        .insert("watch".to_string(), "{invalid: json}".to_string());

    let mut plugin = WatchPlugin::new();
    let result = plugin.on_graph_build(&mut graph).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_watch_plugin_missing_patterns() -> Result<()> {
    let mut graph = Graph::new();
    let mut plugin = WatchPlugin::new();

    // Test with node that has no watch metadata
    let result = plugin.on_graph_build(&mut graph).await;
    assert!(result.is_ok());
    Ok(())
}
