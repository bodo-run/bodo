use bodo::{
    graph::{Graph, NodeKind, TaskData},
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
    let config = PluginConfig {
        options: serde_json::json!({
            "prefix": "[Test] "
        })
        .as_object()
        .cloned(),
    };
    plugin.on_init(&config).await.unwrap();

    let mut graph = Graph::new();
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "task1".to_string(),
        description: None,
        command: Some("echo test".to_string()),
        is_default: false,
        script_name: Some("Test".to_string()),
        working_dir: None,
    }));

    plugin.on_graph_build(&mut graph).await.unwrap();
    let prefix = &graph.nodes[task_id as usize].metadata["prefix"];
    assert_eq!(prefix, "[Test] ");
}
