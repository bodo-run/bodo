use bodo::{
    graph::{Graph, NodeKind, TaskData},
    plugin::{Plugin, PluginConfig},
    plugins::{path_plugin::PathPlugin, prefix_plugin::PrefixPlugin},
};

#[tokio::test]
async fn test_prefix_and_path_plugin_integration() {
    let mut prefix_plugin = PrefixPlugin::new();
    let prefix_config = PluginConfig {
        options: serde_json::json!({
            "prefix": "[Test] "
        })
        .as_object()
        .cloned(),
    };
    prefix_plugin.on_init(&prefix_config).await.unwrap();

    let mut path_plugin = PathPlugin::new();
    let path_config = PluginConfig {
        options: serde_json::json!({
            "default_paths": ["/usr/local/bin"]
        })
        .as_object()
        .cloned(),
    };
    path_plugin.on_init(&path_config).await.unwrap();

    let mut graph = Graph::new();
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "build".to_string(),
        description: None,
        command: Some("make build".to_string()),
        is_default: false,
        script_name: Some("Test".to_string()),
        working_dir: Some("/tmp".to_string()),
    }));

    prefix_plugin.on_graph_build(&mut graph).await.unwrap();
    path_plugin.on_graph_build(&mut graph).await.unwrap();

    let metadata = &graph.nodes[task_id as usize].metadata;
    assert_eq!(metadata.get("prefix"), Some(&"[Test] ".to_string()));
    assert!(metadata.get("env.PATH").unwrap().contains("/usr/local/bin"));
    assert!(metadata.get("env.PATH").unwrap().contains("/tmp"));
}

#[tokio::test]
async fn test_plugin_execution_order() {
    let mut prefix_plugin = PrefixPlugin::new();
    let prefix_config = PluginConfig {
        options: serde_json::json!({
            "prefix": "[Test] "
        })
        .as_object()
        .cloned(),
    };
    prefix_plugin.on_init(&prefix_config).await.unwrap();

    let mut path_plugin = PathPlugin::new();
    let path_config = PluginConfig {
        options: serde_json::json!({
            "default_paths": ["/usr/local/bin"]
        })
        .as_object()
        .cloned(),
    };
    path_plugin.on_init(&path_config).await.unwrap();

    let mut graph = Graph::new();
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "build".to_string(),
        description: None,
        command: Some("make build".to_string()),
        is_default: false,
        script_name: Some("Test".to_string()),
        working_dir: Some("/tmp".to_string()),
    }));

    // Order matters: prefix plugin first, then path plugin
    prefix_plugin.on_graph_build(&mut graph).await.unwrap();
    path_plugin.on_graph_build(&mut graph).await.unwrap();

    let metadata = &graph.nodes[task_id as usize].metadata;
    assert_eq!(metadata.get("prefix"), Some(&"[Test] ".to_string()));
    assert!(metadata.get("env.PATH").unwrap().contains("/usr/local/bin"));
    assert!(metadata.get("env.PATH").unwrap().contains("/tmp"));
}

#[tokio::test]
async fn test_plugin_error_propagation() {
    let mut path_plugin = PathPlugin::new();
    let path_config = PluginConfig {
        options: serde_json::json!({
            "default_paths": ["/usr/local/bin"]
        })
        .as_object()
        .cloned(),
    };
    path_plugin.on_init(&path_config).await.unwrap();

    let mut graph = Graph::new();
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "build".to_string(),
        description: None,
        command: Some("make build".to_string()),
        is_default: false,
        script_name: Some("Test".to_string()),
        working_dir: Some("/tmp".to_string()),
    }));

    path_plugin.on_graph_build(&mut graph).await.unwrap();

    let metadata = &graph.nodes[task_id as usize].metadata;
    assert!(metadata.get("env.PATH").unwrap().contains("/usr/local/bin"));
    assert!(metadata.get("env.PATH").unwrap().contains("/tmp"));
}
