use bodo::graph::{Graph, NodeKind, TaskData};
use bodo::plugin::Plugin;
use bodo::plugins::concurrent_plugin::ConcurrentPlugin;
use std::collections::HashMap;

#[test]
fn test_concurrent_plugin_fallback() {
    let mut plugin = ConcurrentPlugin::new();
    let mut graph = Graph::new();

    // Create a main task
    let task_data_main = TaskData {
        name: "main_task".to_string(),
        description: None,
        command: None,
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
        is_default: true,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
    };
    let main_task_id = graph.add_node(NodeKind::Task(task_data_main));
    let main_node = &mut graph.nodes[main_task_id as usize];
    // Set the metadata 'concurrently' to a JSON string "test_task"
    main_node
        .metadata
        .insert("concurrently".to_string(), "\"test_task\"".to_string());

    // Add a fallback task in task registry with key "prefix test_task"
    let fallback_task = TaskData {
        name: "fallback".to_string(),
        description: None,
        command: Some("echo fallback".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
        is_default: false,
        script_id: "script2".to_string(),
        script_display_name: "script2".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
    };
    let fallback_id = graph.add_node(NodeKind::Task(fallback_task));
    graph
        .task_registry
        .insert("prefix test_task".to_string(), fallback_id);

    let result = plugin.on_graph_build(&mut graph);
    assert!(result.is_ok());

    // Retrieve the concurrent group added by the plugin and check that fallback_id is included.
    let group_nodes: Vec<_> = graph
        .nodes
        .iter()
        .filter(|node| matches!(node.kind, NodeKind::ConcurrentGroup(_)))
        .collect();
    assert_eq!(group_nodes.len(), 1, "Expected one concurrent group node");
    if let NodeKind::ConcurrentGroup(ref group_data) = group_nodes[0].kind {
        assert!(group_data.child_nodes.contains(&fallback_id));
    } else {
        panic!("Expected a concurrent group node");
    }
}
