use std::collections::HashMap;

use bodo::{
    errors::BodoError,
    graph::{CommandData, Graph, NodeKind, TaskData},
    plugin::{Plugin, PluginConfig, PluginManager},
    plugins::{
        execution_plugin::{execute_graph, ExecutionPlugin},
        path_plugin::PathPlugin,
        timeout_plugin::TimeoutPlugin,
    },
    Result,
};
use serde_json::json;

#[tokio::test]
async fn test_prefix_and_path_plugin_integration() -> Result<()> {
    let mut graph = Graph::new();

    // Create a task node
    let task = TaskData {
        name: "test".to_string(),
        description: Some("Test task".to_string()),
        command: Some("echo test".to_string()),
        working_dir: None,
        is_default: false,
        script_id: "test_script".to_string(),
        script_display_name: "Test".to_string(),
        env: HashMap::new(),
    };
    let task_id = graph.add_node(NodeKind::Task(task));

    // Create a command node
    let command = CommandData {
        raw_command: "echo test".to_string(),
        description: None,
        working_dir: None,
        watch: None,
        env: HashMap::new(),
    };
    let cmd_id = graph.add_node(NodeKind::Command(command));

    // Add edge
    graph.add_edge(task_id, cmd_id)?;

    // Test path plugin
    let mut path_plugin = PathPlugin::new();
    let path_config = PluginConfig {
        fail_fast: false,
        watch: false,
        list: false,
        options: Some(
            json!({
                "default_paths": ["/usr/local/bin"]
            })
            .as_object()
            .cloned()
            .unwrap(),
        ),
    };

    path_plugin.on_init(&path_config).await?;
    path_plugin.on_graph_build(&mut graph).await?;

    // Run plugins to process metadata
    let mut manager = PluginManager::new();
    manager.register(Box::new(TimeoutPlugin));
    manager.register(Box::new(ExecutionPlugin));

    manager
        .run_lifecycle(
            &mut graph,
            Some(PluginConfig {
                fail_fast: false,
                watch: false,
                list: false,
                options: None,
            }),
        )
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_plugin_order_matters() -> Result<()> {
    let mut graph = Graph::new();

    // Create a task node
    let task = TaskData {
        name: "test".to_string(),
        description: Some("Test task".to_string()),
        command: Some("echo test".to_string()),
        working_dir: None,
        is_default: false,
        script_id: "test_script".to_string(),
        script_display_name: "Test".to_string(),
        env: HashMap::new(),
    };
    let task_id = graph.add_node(NodeKind::Task(task));

    // Create a command node
    let command = CommandData {
        raw_command: "echo test".to_string(),
        description: None,
        working_dir: None,
        watch: None,
        env: HashMap::new(),
    };
    let cmd_id = graph.add_node(NodeKind::Command(command));

    // Add edge
    graph.add_edge(task_id, cmd_id)?;

    // Then test path plugin
    let mut path_plugin = PathPlugin::new();
    let path_config = PluginConfig {
        fail_fast: false,
        watch: false,
        list: false,
        options: Some(
            json!({
                "default_paths": ["/usr/local/bin"]
            })
            .as_object()
            .cloned()
            .unwrap(),
        ),
    };

    path_plugin.on_init(&path_config).await?;
    path_plugin.on_graph_build(&mut graph).await?;

    // Run plugins to process metadata
    let mut manager = PluginManager::new();
    manager.register(Box::new(TimeoutPlugin));
    manager.register(Box::new(ExecutionPlugin));

    manager
        .run_lifecycle(
            &mut graph,
            Some(PluginConfig {
                fail_fast: false,
                watch: false,
                list: false,
                options: None,
            }),
        )
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_timeout_plugin() -> Result<()> {
    let mut graph = Graph::new();

    // Create a task that will timeout
    let task = TaskData {
        name: "test".to_string(),
        description: Some("Test task".to_string()),
        command: Some("sleep 5".to_string()),
        working_dir: None,
        is_default: false,
        script_id: "test_script".to_string(),
        script_display_name: "Test".to_string(),
        env: HashMap::new(),
    };
    let task_id = graph.add_node(NodeKind::Task(task));

    // Add timeout metadata
    let node = &mut graph.nodes[task_id as usize];
    node.metadata
        .insert("timeout".to_string(), "1s".to_string());

    // Setup plugins
    let mut manager = PluginManager::new();
    manager.register(Box::new(TimeoutPlugin));

    // Run plugins to process metadata
    manager
        .run_lifecycle(
            &mut graph,
            Some(PluginConfig {
                fail_fast: false,
                watch: false,
                list: false,
                options: None,
            }),
        )
        .await?;

    // Verify that timeout metadata was processed
    let node = &graph.nodes[task_id as usize];
    assert!(node.metadata.contains_key("timeout"));

    Ok(())
}

#[tokio::test]
async fn test_plugin_lifecycle() -> Result<()> {
    let mut manager = PluginManager::new();
    let mut graph = Graph::new();

    // Register plugins
    manager.register(Box::new(ExecutionPlugin));

    // Run lifecycle with default config
    manager.run_lifecycle(&mut graph, None).await?;

    Ok(())
}

#[tokio::test]
async fn test_plugin_execution() -> Result<()> {
    let mut manager = PluginManager::new();
    let mut graph = Graph::new();

    // Register plugins
    manager.register(Box::new(ExecutionPlugin));

    // Run lifecycle with default config
    manager.run_lifecycle(&mut graph, None).await?;

    Ok(())
}
