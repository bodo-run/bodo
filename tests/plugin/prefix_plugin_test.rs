use bodo::{
    graph::{Graph, NodeKind, TaskData},
    plugin::{Plugin, PluginConfig},
    plugins::prefix_plugin::PrefixPlugin,
    Result,
};
use serde_json::json;
use std::collections::HashMap;

#[tokio::test]
async fn test_prefix_plugin_default_config() -> Result<()> {
    let mut plugin = PrefixPlugin::new();
    plugin.on_init(&PluginConfig::default()).await?;

    let mut graph = Graph::new();
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "test".to_string(),
        description: Some("Test task".to_string()),
        command: Some("echo test".to_string()),
        working_dir: None,
        is_default: false,
        script_name: Some("Test".to_string()),
        env: HashMap::new(),
    }));

    plugin.on_graph_build(&mut graph).await?;

    if let NodeKind::Task(task) = &graph.nodes[task_id as usize].kind {
        assert_eq!(task.description, Some("Test task".to_string()));
    } else {
        panic!("Expected Task node");
    }

    Ok(())
}

#[tokio::test]
async fn test_prefix_plugin_custom_config() -> Result<()> {
    let mut plugin = PrefixPlugin::new();
    let config = PluginConfig {
        options: Some(
            json!({
                "prefix": "[Test] "
            })
            .as_object()
            .cloned()
            .unwrap(),
        ),
    };
    plugin.on_init(&config).await?;

    let mut graph = Graph::new();
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "test".to_string(),
        description: Some("Test task".to_string()),
        command: Some("echo test".to_string()),
        working_dir: None,
        is_default: false,
        script_name: Some("Test".to_string()),
        env: HashMap::new(),
    }));

    plugin.on_graph_build(&mut graph).await?;

    if let NodeKind::Task(task) = &graph.nodes[task_id as usize].kind {
        assert_eq!(task.description, Some("[Test] Test task".to_string()));
    } else {
        panic!("Expected Task node");
    }

    Ok(())
}
