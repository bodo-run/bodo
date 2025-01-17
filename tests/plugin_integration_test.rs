use std::collections::HashMap;

use bodo::{
    graph::{Graph, NodeKind, TaskData},
    plugin::{Plugin, PluginConfig},
    plugins::{path_plugin::PathPlugin, prefix_plugin::PrefixPlugin},
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
