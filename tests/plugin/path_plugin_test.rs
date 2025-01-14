use bodo::{
    graph::{CommandData, Graph, NodeKind, TaskData},
    plugin::{Plugin, PluginConfig},
    plugins::path_plugin::PathPlugin,
};
use std::path::PathBuf;

#[tokio::test]
async fn test_path_plugin_on_init_no_paths() {
    let mut plugin = PathPlugin::new();
    let config = PluginConfig { options: None };
    let result = plugin.on_init(&config).await;
    assert!(result.is_ok());
    // Should remain empty
    assert!(plugin.default_paths.is_empty());
}

#[tokio::test]
async fn test_path_plugin_on_init_with_paths() {
    let mut plugin = PathPlugin::new();
    let config = PluginConfig {
        options: serde_json::json!({
            "default_paths": ["/usr/local/bin", "/custom/bin"]
        })
        .as_object()
        .cloned(),
    };
    let result = plugin.on_init(&config).await;
    assert!(result.is_ok());
    assert_eq!(
        plugin.default_paths,
        vec![
            PathBuf::from("/usr/local/bin"),
            PathBuf::from("/custom/bin")
        ]
    );
}

#[tokio::test]
async fn test_path_plugin_on_graph_build() {
    let mut plugin = PathPlugin::new();
    plugin.default_paths = vec![PathBuf::from("/usr/local/bin")];

    let mut graph = Graph::new();
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "build".to_string(),
        description: None,
        command: Some("make build".to_string()),
        working_dir: Some("/tmp".to_string()),
    }));
    let cmd_id = graph.add_node(NodeKind::Command(CommandData {
        raw_command: "echo Hello".to_string(),
        description: Some("Echo command".to_string()),
        working_dir: None,
    }));

    // Insert some metadata that the plugin will interpret
    graph.nodes[task_id as usize]
        .metadata
        .insert("exec_paths".to_string(), "[\"/opt/bin\"]".to_string());

    let result = plugin.on_graph_build(&mut graph).await;
    assert!(result.is_ok());

    let task_env_path = &graph.nodes[task_id as usize].metadata["env.PATH"];
    let cmd_env_path = &graph.nodes[cmd_id as usize].metadata["env.PATH"];

    // On most Unix systems, path separator is ":", but we read from PATH_SEPARATOR
    // environment variable or default to ":". For the test, we just check substrings.
    assert!(task_env_path.contains("/usr/local/bin"));
    assert!(task_env_path.contains("/opt/bin"));
    assert!(task_env_path.contains("/tmp")); // working_dir
    assert!(cmd_env_path.contains("/usr/local/bin"));
    // The command node had no exec_paths, so only default_paths apply
    assert!(!cmd_env_path.contains("/opt/bin"));
}
