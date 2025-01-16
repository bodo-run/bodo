use bodo::{
    graph::{CommandData, Graph, NodeKind, TaskData},
    plugin::{Plugin, PluginConfig},
    plugins::path_plugin::PathPlugin,
};

#[tokio::test]
async fn test_path_plugin_with_task() {
    let mut plugin = PathPlugin::new();
    let mut graph = Graph::new();

    let task_data = TaskData {
        name: "test".to_string(),
        description: None,
        command: Some("echo hello".to_string()),
        is_default: false,
        script_name: None,
        working_dir: Some("/tmp".to_string()),
    };

    let node_id = graph.add_node(NodeKind::Task(task_data));
    plugin.on_graph_build(&mut graph).await.unwrap();

    if let NodeKind::Task(task_data) = &graph.nodes[node_id as usize].kind {
        assert_eq!(task_data.working_dir, Some("/tmp".to_string()));
    }
}

#[tokio::test]
async fn test_path_plugin_with_command() {
    let mut plugin = PathPlugin::new();
    let mut graph = Graph::new();

    let cmd_data = CommandData {
        raw_command: "echo hello".to_string(),
        description: None,
        watch: None,
        working_dir: None,
    };

    let node_id = graph.add_node(NodeKind::Command(cmd_data));
    plugin.on_graph_build(&mut graph).await.unwrap();

    if let NodeKind::Command(cmd_data) = &graph.nodes[node_id as usize].kind {
        assert_eq!(cmd_data.working_dir, None);
    }
}
