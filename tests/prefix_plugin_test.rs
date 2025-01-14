use bodo::{
    graph::{CommandData, Graph, NodeKind, TaskData},
    plugin::{Plugin, PluginConfig},
    plugins::prefix_plugin::PrefixPlugin,
};

#[tokio::test]
async fn test_prefix_plugin_on_init_no_options() {
    let mut plugin = PrefixPlugin::new();
    let config = PluginConfig {
        options: None, // no prefix specified
    };
    let result = plugin.on_init(&config).await;
    assert!(result.is_ok());
    // Should remain the default "[{}]"
    assert_eq!(plugin.prefix_format, "[{}]");
}

#[tokio::test]
async fn test_prefix_plugin_on_init_with_options() {
    let mut plugin = PrefixPlugin::new();
    let config = PluginConfig {
        options: serde_json::json!({
            "prefix_format": "<<{}>>"
        })
        .as_object()
        .cloned(),
    };
    let result = plugin.on_init(&config).await;
    assert!(result.is_ok());
    // Should update to "<<{}>>"
    assert_eq!(plugin.prefix_format, "<<{}>>");
}

#[tokio::test]
async fn test_prefix_plugin_on_graph_build() {
    let mut plugin = PrefixPlugin::new();
    plugin.prefix_format = "[task-{}]".to_string();

    let mut graph = Graph::new();
    // Add a Task node
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "build".to_string(),
        description: Some("Build something".to_string()),
        command: None,
        working_dir: None,
    }));
    // Add a Command node
    let cmd_id = graph.add_node(NodeKind::Command(CommandData {
        raw_command: "echo Test".to_string(),
        description: Some("Test command".to_string()),
        working_dir: None,
        watch: None,
    }));

    // Run the plugin
    let result = plugin.on_graph_build(&mut graph).await;
    assert!(result.is_ok());

    // Check if metadata is inserted as expected
    let task_metadata = &graph.nodes[task_id as usize].metadata;
    let cmd_metadata = &graph.nodes[cmd_id as usize].metadata;

    assert_eq!(
        task_metadata.get("prefix"),
        Some(&"[task-build]".to_string())
    );

    // The command node prefix is derived from the first token of raw_command
    // e.g. "echo"
    assert_eq!(cmd_metadata.get("prefix"), Some(&"[task-echo]".to_string()));
}
