use bodo::graph::{Graph, NodeKind, TaskData};
use bodo::plugin::Plugin;
use bodo::plugins::timeout_plugin::TimeoutPlugin;
use std::collections::HashMap;

#[test]
fn test_timeout_plugin_on_graph_build() {
    let mut plugin = TimeoutPlugin::new();
    let mut graph = Graph::new();

    let mut metadata = HashMap::new();
    metadata.insert("timeout".to_string(), "30s".to_string());

    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "test_task".to_string(),
        description: None,
        command: None,
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    }));
    graph.nodes[task_id as usize].metadata = metadata;

    let result = plugin.on_graph_build(&mut graph);
    assert!(result.is_ok());

    let node = &graph.nodes[task_id as usize];
    assert_eq!(
        node.metadata.get("timeout_seconds"),
        Some(&"30".to_string())
    );
}

#[test]
fn test_timeout_plugin_invalid_timeout() {
    let mut plugin = TimeoutPlugin::new();
    let mut graph = Graph::new();

    let mut metadata = HashMap::new();
    metadata.insert("timeout".to_string(), "invalid".to_string());

    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "test_task".to_string(),
        description: None,
        command: None,
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    }));
    graph.nodes[task_id as usize].metadata = metadata;

    let result = plugin.on_graph_build(&mut graph);
    assert!(result.is_err());
}
