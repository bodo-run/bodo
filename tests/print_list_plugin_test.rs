use bodo::graph::{Graph, NodeKind, TaskData};
use bodo::plugin::Plugin;
use bodo::plugins::print_list_plugin::PrintListPlugin;
use std::collections::HashMap;

#[test]
fn test_print_list_plugin() {
    let mut plugin = PrintListPlugin;

    let mut graph = Graph::new();

    // Create two tasks with different properties
    let task_data1 = TaskData {
        name: "task1".to_string(),
        description: Some("First task".to_string()),
        command: Some("echo Task 1".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "script1".to_string(),
        script_display_name: "Script 1".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    };

    let task_data2 = TaskData {
        name: "task2".to_string(),
        description: Some("Second task".to_string()),
        command: Some("echo Task 2".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "script2".to_string(),
        script_display_name: "Script 2".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    };

    // Add nodes to graph
    graph.add_node(NodeKind::Task(Box::new(task_data1)));
    graph.add_node(NodeKind::Task(Box::new(task_data2)));

    // Apply the plugin
    let result = plugin.on_graph_build(&mut graph);
    assert!(result.is_ok(), "Plugin execution failed");
}
