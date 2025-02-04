use bodo::errors::BodoError;
use bodo::graph::{Graph, NodeKind, TaskData};
use bodo::plugins::concurrent_plugin::ConcurrentPlugin;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_concurrent_plugin_invalid_object_dependency() {
    let mut plugin = ConcurrentPlugin::new();
    let mut graph = Graph::new();

    let task_data_main = TaskData {
        name: "main_task".to_string(),
        description: None,
        command: None,
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: true,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    };

    let main_task_id = graph.add_node(NodeKind::Task(task_data_main));

    // Create an invalid concurrently meta: object with no "task" or "command" key
    let invalid_obj = json!({"invalid_key": "value"});
    let meta_str = serde_json::to_string(&invalid_obj).unwrap();

    let main_node = &mut graph.nodes[main_task_id as usize];
    main_node
        .metadata
        .insert("concurrently".to_string(), meta_str);

    let result = plugin.on_graph_build(&mut graph);
    match result {
        Err(BodoError::PluginError(msg)) => {
            assert!(
                msg.contains("Invalid concurrency dependency format"),
                "Unexpected error message: {}",
                msg
            );
        }
        _ => panic!("Expected PluginError due to invalid dependency object"),
    }
}
