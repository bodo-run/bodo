use std::collections::HashMap;

use bodo::{
    graph::{CommandData, Graph, NodeKind, TaskData},
    plugin::{Plugin, PluginConfig, PluginManager},
    plugins::path_plugin::PathPlugin,
    Result,
};
use serde_json::json;

#[tokio::test]
async fn test_path_plugin() -> Result<()> {
    let mut graph = Graph::new();

    // Create a task with a working directory
    let task = TaskData {
        name: "test".to_string(),
        description: Some("Test task".to_string()),
        command: Some("echo test".to_string()),
        working_dir: Some("/tmp".to_string()),
        is_default: false,
        script_id: "test_script".to_string(),
        script_display_name: "Test".to_string(),
        env: HashMap::new(),
    };
    let task_id = graph.add_node(NodeKind::Task(task));

    // Setup plugins
    let mut manager = PluginManager::new();
    manager.register(Box::new(PathPlugin::new()));

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

    // Verify working directory was processed
    let node = &graph.nodes[task_id as usize];
    if let NodeKind::Task(task_data) = &node.kind {
        assert_eq!(task_data.working_dir, Some("/tmp".to_string()));
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
        fail_fast: false,
        watch: false,
        list: false,
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
