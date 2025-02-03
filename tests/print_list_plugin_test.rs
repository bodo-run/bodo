use bodo::graph::{Graph, NodeKind, TaskData};
use bodo::plugin::Plugin;
use bodo::plugins::print_list_plugin::PrintListPlugin;
use env_logger::Builder;
use log::LevelFilter;
use std::collections::HashMap;

use std::sync::Once;
static INIT: Once = Once::new();

fn init_logger() {
    INIT.call_once(|| {
        // Initialize the logger for capturing test output
        Builder::new()
            .filter_level(LevelFilter::Info)
            .is_test(true)
            .init();
    });
}

#[test]
fn test_print_list_plugin_on_graph_build() {
    init_logger();
    let mut plugin = PrintListPlugin;
    let mut graph = Graph::new();

    // Create some tasks
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

    // Since the plugin prints to log, and we cannot easily capture it here,
    // we can ensure that the code runs without errors for coverage.
}
