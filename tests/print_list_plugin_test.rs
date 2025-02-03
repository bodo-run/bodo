use bodo::graph::{Graph, NodeKind, TaskData};
use bodo::plugin::Plugin;
use bodo::plugins::print_list_plugin::PrintListPlugin;
use std::collections::HashMap;

#[test]
fn test_print_list_plugin_on_graph_build() {
    let mut plugin = PrintListPlugin;
    let mut graph = Graph::new();

    // Create tasks with different script display names
    let task1_id = graph.add_node(NodeKind::Task(TaskData {
        name: "task1".to_string(),
        description: Some("Description of task1".to_string()),
        command: Some("echo Task 1".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        is_default: true,
        script_id: "script1.yaml".to_string(),
        script_display_name: "Script1".to_string(),
        watch: None,
    }));

    let task2_id = graph.add_node(NodeKind::Task(TaskData {
        name: "task2".to_string(),
        description: Some("Description of task2".to_string()),
        command: Some("echo Task 2".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        is_default: false,
        script_id: "script2.yaml".to_string(),
        script_display_name: "Script2".to_string(),
        watch: None,
    }));

    // Run the plugin
    let result = plugin.on_graph_build(&mut graph);
    assert!(result.is_ok());
}
