use std::collections::HashMap;

use bodo::{
    errors::BodoError,
    graph::{CommandData, Graph, NodeKind, TaskData},
    plugin::{Plugin, PluginConfig, PluginManager},
    plugins::{
        execution_plugin::{execute_graph, ExecutionPlugin},
        path_plugin::PathPlugin,
        prefix_plugin::PrefixPlugin,
        timeout_plugin::TimeoutPlugin,
    },
    Result,
};
use serde_json::json;
use tempfile::tempdir;

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
        script_name: Some("Test".to_string()),
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

    // Test prefix plugin
    let mut prefix_plugin = PrefixPlugin::new();
    let prefix_config = PluginConfig {
        fail_fast: false,
        watch: false,
        list: false,
        options: Some(
            json!({
                "prefix": "[Test] "
            })
            .as_object()
            .cloned()
            .unwrap(),
        ),
    };

    prefix_plugin.on_init(&prefix_config).await?;
    prefix_plugin.on_graph_build(&mut graph).await?;

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
        script_name: Some("Test".to_string()),
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

    // Test prefix plugin first
    let mut prefix_plugin = PrefixPlugin::new();
    let prefix_config = PluginConfig {
        fail_fast: false,
        watch: false,
        list: false,
        options: Some(
            json!({
                "prefix": "[Test] "
            })
            .as_object()
            .cloned()
            .unwrap(),
        ),
    };

    prefix_plugin.on_init(&prefix_config).await?;
    prefix_plugin.on_graph_build(&mut graph).await?;

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
        script_name: Some("Test".to_string()),
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
    manager.register(Box::new(ExecutionPlugin));

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

    // Execute the graph
    let result = execute_graph(&mut graph).await;

    // Verify timeout error
    assert!(result.is_err());
    assert!(matches!(result, Err(BodoError::PluginError(_))));

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
