use bodo::{
    errors::BodoError,
    graph::{Graph, NodeKind, TaskData},
    plugin::{Plugin, PluginConfig, PluginManager},
    plugins::watch_plugin::WatchPlugin,
    Result,
};
use serde_json::json;
use std::{collections::HashMap, fs};
use tempfile::tempdir;

#[tokio::test]
async fn test_watch_plugin() -> Result<()> {
    let temp_dir = tempdir()?;
    let src_dir = temp_dir.path().join("src");
    fs::create_dir_all(&src_dir)?;
    fs::write(src_dir.join("main.rs"), "fn main() {}")?;

    let mut graph = Graph::new();

    // Create a task
    let task = TaskData {
        name: "test".to_string(),
        description: Some("Test task".to_string()),
        command: Some("echo 'File changed'".to_string()),
        working_dir: Some(temp_dir.path().to_string_lossy().into_owned()),
        env: HashMap::new(),
        is_default: false,
        script_id: "test_script".to_string(),
        script_display_name: "Test".to_string(),
    };
    let task_id = graph.add_node(NodeKind::Task(task));

    // Add valid watch configuration
    let node = &mut graph.nodes[task_id as usize];
    node.metadata.insert(
        "watch".to_string(),
        json!({
            "patterns": [src_dir.join("**/*.rs").to_string_lossy().to_string()],
            "debounce_ms": 100,
            "ignore_patterns": ["target/**/*"]
        })
        .to_string(),
    );

    // Setup plugins
    let mut manager = PluginManager::new();
    manager.register(Box::new(WatchPlugin::new()));

    // Run plugins to process metadata
    manager
        .run_lifecycle(
            &mut graph,
            Some(PluginConfig {
                fail_fast: false,
                watch: true,
                list: false,
                options: None,
            }),
        )
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_watch_plugin_invalid_config() -> Result<()> {
    let mut graph = Graph::new();

    // Create a task
    let task = TaskData {
        name: "test".to_string(),
        description: Some("Test task".to_string()),
        command: Some("echo 'Should fail'".to_string()),
        working_dir: None,
        env: HashMap::new(),
        is_default: false,
        script_id: "test_script".to_string(),
        script_display_name: "Test".to_string(),
    };
    let task_id = graph.add_node(NodeKind::Task(task));

    // Add invalid JSON configuration
    let node = &mut graph.nodes[task_id as usize];
    node.metadata
        .insert("watch".to_string(), "invalid json".to_string());

    // Setup plugins
    let mut manager = PluginManager::new();
    manager.register(Box::new(WatchPlugin::new()));

    // Run plugins to process metadata
    let result = manager
        .run_lifecycle(
            &mut graph,
            Some(PluginConfig {
                fail_fast: false,
                watch: true,
                list: false,
                options: None,
            }),
        )
        .await;

    assert!(result.is_err());
    assert!(matches!(result, Err(BodoError::PluginError(_))));

    Ok(())
}

#[tokio::test]
async fn test_watch_plugin_missing_patterns() -> Result<()> {
    let mut graph = Graph::new();
    let mut plugin = WatchPlugin::new();

    // Create a task without watch metadata
    let task = TaskData {
        name: "test".to_string(),
        description: Some("Test task".to_string()),
        command: Some("echo test".to_string()),
        working_dir: None,
        env: HashMap::new(),
        is_default: false,
        script_id: "test_script".to_string(),
        script_display_name: "Test".to_string(),
    };
    let _task_id = graph.add_node(NodeKind::Task(task));

    // Run plugin
    let result = plugin.on_graph_build(&mut graph).await;
    assert!(result.is_ok());

    Ok(())
}
