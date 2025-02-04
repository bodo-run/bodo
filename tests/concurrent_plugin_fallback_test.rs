use bodo::graph::{Graph, NodeKind, TaskData};
use bodo::plugins::concurrent_plugin::ConcurrentPlugin;
use serde_json::Value;
use std::collections::HashMap;

#[test]
fn test_concurrent_plugin_fallback_search() {
    let mut graph = Graph::new();
    // Create a main task with "concurrently" metadata set to a simple string "test_task"
    let main_task = TaskData {
        name: "main".to_string(),
        description: None,
        command: Some("echo main".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: true,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    };
    let main_id = graph.add_node(NodeKind::Task(main_task));
    // Set concurrently metadata to a JSON string representing "test_task"
    graph.nodes[main_id as usize]
        .metadata
        .insert("concurrently".to_string(), "\"test_task\"".to_string());

    // Add a fallback task with registry key "prefix test_task"
    let fallback_task = TaskData {
        name: "fallback".to_string(),
        description: None,
        command: Some("echo fallback".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "script2".to_string(),
        script_display_name: "script2".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    };
    let fallback_id = graph.add_node(NodeKind::Task(fallback_task));
    graph
        .task_registry
        .insert("prefix test_task".to_string(), fallback_id);

    let mut plugin = ConcurrentPlugin::new();
    let res = plugin.on_graph_build(&mut graph);
    assert!(res.is_ok());

    // Retrieve the concurrent group added by the plugin and check that fallback_id is included.
    let group_nodes: Vec<_> = graph
        .nodes
        .iter()
        .filter(|node| matches!(node.kind, NodeKind::ConcurrentGroup(_)))
        .collect();
    assert_eq!(group_nodes.len(), 1);
    if let NodeKind::ConcurrentGroup(ref group_data) = group_nodes[0].kind {
        assert!(group_data.child_nodes.contains(&fallback_id));
    } else {
        panic!("Expected a concurrent group node");
    }
}
