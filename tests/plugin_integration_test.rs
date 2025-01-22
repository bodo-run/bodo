use std::collections::HashMap;

use bodo::{
    graph::{Graph, NodeKind, TaskData},
    plugin::{Plugin, PluginConfig, PluginManager},
    plugins::{
        execution_plugin::{self, ExecutionPlugin},
        path_plugin::PathPlugin,
        prefix_plugin::PrefixPlugin,
        timeout_plugin::TimeoutPlugin,
    },
    Result,
};
use serde_json::json;

#[tokio::test]
async fn test_prefix_and_path_plugin_integration() -> Result<()> {
    // Initialize plugins
    let mut prefix_plugin = PrefixPlugin::new();
    let prefix_config = PluginConfig {
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

    let mut path_plugin = PathPlugin::new();
    let path_config = PluginConfig {
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

    // Create a test graph
    let mut graph = Graph::new();
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "build".to_string(),
        description: Some("Build the project".to_string()),
        command: Some("make build".to_string()),
        working_dir: Some("/tmp".to_string()),
        is_default: false,
        script_name: Some("Test".to_string()),
        env: HashMap::new(),
    }));

    // Apply plugins in sequence
    prefix_plugin.on_graph_build(&mut graph).await?;
    path_plugin.on_graph_build(&mut graph).await?;

    // Verify the results
    let node = &graph.nodes[task_id as usize];
    if let NodeKind::Task(task) = &node.kind {
        assert_eq!(
            task.description,
            Some("[Test] Build the project".to_string())
        );
        assert!(task.env.get("PATH").unwrap().contains("/usr/local/bin"));
        assert!(task.env.get("PATH").unwrap().contains("/tmp"));
    } else {
        panic!("Expected task node");
    }

    Ok(())
}

#[tokio::test]
async fn test_plugin_order_matters() -> Result<()> {
    // Initialize plugins
    let mut prefix_plugin = PrefixPlugin::new();
    let prefix_config = PluginConfig {
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

    let mut path_plugin = PathPlugin::new();
    let path_config = PluginConfig {
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

    // Create a test graph
    let mut graph = Graph::new();
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "build".to_string(),
        description: Some("Build the project".to_string()),
        command: Some("make build".to_string()),
        working_dir: Some("/tmp".to_string()),
        is_default: false,
        script_name: Some("Test".to_string()),
        env: HashMap::new(),
    }));

    // Apply plugins in reverse order
    path_plugin.on_graph_build(&mut graph).await?;
    prefix_plugin.on_graph_build(&mut graph).await?;

    // Verify the results
    let node = &graph.nodes[task_id as usize];
    if let NodeKind::Task(task) = &node.kind {
        assert_eq!(
            task.description,
            Some("[Test] Build the project".to_string())
        );
        assert!(task.env.get("PATH").unwrap().contains("/usr/local/bin"));
        assert!(task.env.get("PATH").unwrap().contains("/tmp"));
    } else {
        panic!("Expected task node");
    }

    Ok(())
}

#[tokio::test]
async fn test_plugin_chain_with_empty_config() -> Result<()> {
    // Initialize plugins with empty configs
    let mut prefix_plugin = PrefixPlugin::new();
    let mut path_plugin = PathPlugin::new();

    prefix_plugin.on_init(&PluginConfig::default()).await?;
    path_plugin.on_init(&PluginConfig::default()).await?;

    // Create a test graph
    let mut graph = Graph::new();
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "build".to_string(),
        description: Some("Build the project".to_string()),
        command: Some("make build".to_string()),
        working_dir: Some("/tmp".to_string()),
        is_default: false,
        script_name: Some("Test".to_string()),
        env: HashMap::new(),
    }));

    // Apply plugins
    prefix_plugin.on_graph_build(&mut graph).await?;
    path_plugin.on_graph_build(&mut graph).await?;

    // Verify the results
    let node = &graph.nodes[task_id as usize];
    if let NodeKind::Task(task) = &node.kind {
        // Description should be unchanged since prefix was empty
        assert_eq!(task.description, Some("Build the project".to_string()));
        // PATH should only contain /tmp since no default paths were set
        if let Some(path) = task.env.get("PATH") {
            assert!(path.contains("/tmp"));
            assert!(!path.contains("/usr/local/bin"));
        }
    } else {
        panic!("Expected task node");
    }

    Ok(())
}

#[tokio::test]
async fn test_timeout_plugin() -> Result<()> {
    let mut graph = Graph::new();
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "timeout_test".to_string(),
        description: Some("Test timeout".to_string()),
        command: Some("sleep 2".to_string()), // Command that runs longer than timeout
        working_dir: None,
        env: HashMap::new(),
        is_default: false,
        script_name: None,
    }));

    // Add timeout metadata (1 second)
    let node = &mut graph.nodes[task_id as usize];
    node.metadata
        .insert("timeout".to_string(), "1s".to_string());

    // Setup plugins
    let mut manager = PluginManager::new();
    manager.register(Box::new(TimeoutPlugin));
    manager.register(Box::new(ExecutionPlugin));

    // Run plugins to process metadata
    manager
        .run_lifecycle(&mut graph, &PluginConfig::default())
        .await?;

    // Execute the graph
    let result = execution_plugin::execute_graph(&mut manager, &mut graph).await;

    // Verify timeout error
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("timed out"), "Error was: {}", err_msg);

    Ok(())
}
