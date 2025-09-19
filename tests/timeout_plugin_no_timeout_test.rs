use bodo::graph::{Graph, NodeKind, TaskData};
use bodo::plugin::Plugin;
use bodo::plugins::timeout_plugin::TimeoutPlugin;
use std::collections::HashMap;

#[test]
fn test_timeout_plugin_no_timeout() {
    let mut plugin = TimeoutPlugin::new();
    let mut graph = Graph::new();
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "no_timeout".to_string(),
        description: Some("Task with no timeout".to_string()),
        command: Some("echo no timeout".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "dummy".to_string(),
        script_display_name: "dummy".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    }));
    // Do not set timeout metadata
    let result = plugin.on_graph_build(&mut graph);
    assert!(result.is_ok());
    // Timeout metadata should not be present.
    let node = &graph.nodes[task_id as usize];
    assert!(!node.metadata.contains_key("timeout_seconds"));
}
