use bodo::{
    graph::{CommandData, Graph, NodeKind, TaskData},
    plugin::{Plugin, PluginConfig},
    plugins::path_plugin::PathPlugin,
};

#[tokio::test]
async fn test_path_plugin_on_init_no_paths() {
    let mut plugin = PathPlugin::new();
    let config = PluginConfig { options: None };
    let result = plugin.on_init(&config).await;
    assert!(result.is_ok());

    // Test through graph build instead of direct field access
    let mut graph = Graph::new();
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "test".to_string(),
        description: None,
        command: Some("echo test".to_string()),
        working_dir: Some("/tmp".to_string()),
        is_default: false,
        script_name: Some("Test".to_string()),
    }));

    plugin.on_graph_build(&mut graph).await.unwrap();
    let path = &graph.nodes[task_id as usize].metadata["env.PATH"];
    assert!(path.contains("/tmp")); // Only working_dir should be in path
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

    // Test through graph build instead of direct field access
    let mut graph = Graph::new();
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "test".to_string(),
        description: None,
        command: Some("echo test".to_string()),
        working_dir: Some("/tmp".to_string()),
        is_default: false,
        script_name: Some("Test".to_string()),
    }));

    plugin.on_graph_build(&mut graph).await.unwrap();
    let path = &graph.nodes[task_id as usize].metadata["env.PATH"];
    assert!(path.contains("/usr/local/bin"));
    assert!(path.contains("/custom/bin"));
    assert!(path.contains("/tmp")); // working_dir should also be included
}

#[tokio::test]
async fn test_path_plugin_on_graph_build() {
    let mut plugin = PathPlugin::new();
    let config = PluginConfig {
        options: serde_json::json!({
            "default_paths": ["/usr/local/bin"]
        })
        .as_object()
        .cloned(),
    };
    plugin.on_init(&config).await.unwrap();

    let mut graph = Graph::new();
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "build".to_string(),
        description: None,
        command: Some("make build".to_string()),
        working_dir: Some("/tmp".to_string()),
        is_default: false,
        script_name: Some("Test".to_string()),
    }));
    let cmd_id = graph.add_node(NodeKind::Command(CommandData {
        raw_command: "echo Hello".to_string(),
        description: Some("Echo command".to_string()),
        working_dir: None,
        watch: None,
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
