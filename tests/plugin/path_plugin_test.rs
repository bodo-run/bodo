use std::collections::HashMap;

use bodo::{
    graph::{CommandData, Graph, NodeKind, TaskData},
    plugin::{Plugin, PluginConfig},
    plugins::path_plugin::PathPlugin,
    Result,
};
use serde_json::json;

#[tokio::test]
async fn test_path_plugin_task() -> Result<()> {
    let task_data = TaskData {
        name: "test".to_string(),
        description: None,
        command: Some("echo test".to_string()),
        working_dir: Some("/tmp".to_string()),
        is_default: false,
        script_name: None,
        env: HashMap::new(),
    };

    let mut graph = Graph::new();
    graph.add_node(NodeKind::Task(task_data));

    let mut plugin = PathPlugin::new();
    let config = PluginConfig {
        options: Some(
            json!({
                "default_paths": ["/usr/local/bin", "/usr/bin"]
            })
            .as_object()
            .cloned()
            .unwrap(),
        ),
    };

    plugin.on_init(&config).await?;
    plugin.on_graph_build(&mut graph).await?;

    if let NodeKind::Task(task) = &graph.nodes[0].kind {
        let path = task.env.get("PATH").expect("PATH should be set");
        assert!(path.contains("/usr/local/bin"));
        assert!(path.contains("/usr/bin"));
        assert!(path.contains("/tmp"));
    } else {
        panic!("Expected Task node");
    }

    Ok(())
}

#[tokio::test]
async fn test_path_plugin_command() -> Result<()> {
    let cmd_data = CommandData {
        raw_command: "echo test".to_string(),
        description: None,
        working_dir: Some("/tmp".to_string()),
        watch: None,
        env: HashMap::new(),
    };

    let mut graph = Graph::new();
    graph.add_node(NodeKind::Command(cmd_data));

    let mut plugin = PathPlugin::new();
    let config = PluginConfig {
        options: Some(
            json!({
                "default_paths": ["/usr/local/bin", "/usr/bin"]
            })
            .as_object()
            .cloned()
            .unwrap(),
        ),
    };

    plugin.on_init(&config).await?;
    plugin.on_graph_build(&mut graph).await?;

    if let NodeKind::Command(cmd) = &graph.nodes[0].kind {
        let path = cmd.env.get("PATH").expect("PATH should be set");
        assert!(path.contains("/usr/local/bin"));
        assert!(path.contains("/usr/bin"));
        assert!(path.contains("/tmp"));
    } else {
        panic!("Expected Command node");
    }

    Ok(())
}
