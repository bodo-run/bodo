use bodo::graph::{ConcurrentGroupData, Graph, NodeKind, TaskData};
use bodo::plugin::Plugin;
use bodo::plugins::prefix_plugin::PrefixPlugin;
use std::collections::HashMap;

#[test]
fn test_prefix_plugin_on_graph_build() {
    let mut plugin = PrefixPlugin::new();
    let mut graph = Graph::new();

    // Create a ConcurrentGroup node with prefix_output metadata
    let group_node_id = graph.add_node(NodeKind::ConcurrentGroup(ConcurrentGroupData {
        child_nodes: vec![],
        fail_fast: true,
        max_concurrent: None,
        timeout_secs: None,
    }));
    graph.nodes[group_node_id as usize]
        .metadata
        .insert("prefix_output".to_string(), "true".to_string());

    // Add child tasks
    let task1_id = graph.add_node(NodeKind::Task(TaskData {
        name: "task1".to_string(),
        description: None,
        command: Some("echo Task 1".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    }));

    let task2_id = graph.add_node(NodeKind::Task(TaskData {
        name: "task2".to_string(),
        description: None,
        command: Some("echo Task 2".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    }));

    // For the group node, update each child by adding prefix metadata if needed.
    let user_color: Option<String> = None;
    for &child_id in &[task1_id, task2_id] {
        let child_node = &graph.nodes[child_id as usize];
        let (label, default_color) = match &child_node.kind {
            NodeKind::Task(t) => (t.name.clone(), plugin.next_color()),
            NodeKind::Command(_) => (format!("cmd-{}", child_id), plugin.next_color()),
            NodeKind::ConcurrentGroup(_) => (format!("group-{}", child_id), plugin.next_color()),
        };
        let chosen_color = user_color.clone().unwrap_or(default_color);
        // (Simulate updating metadata if needed; this test primarily checks on_graph_build)
    }
    let result = plugin.on_graph_build(&mut graph);
    assert!(result.is_ok());

    // Check that child tasks have prefix metadata inserted.
    let child1 = &graph.nodes[task1_id as usize];
    assert_eq!(
        child1.metadata.get("prefix_enabled"),
        Some(&"true".to_string())
    );
    assert!(child1.metadata.get("prefix_label").is_some());
    assert!(child1.metadata.get("prefix_color").is_some());

    let child2 = &graph.nodes[task2_id as usize];
    assert_eq!(
        child2.metadata.get("prefix_enabled"),
        Some(&"true".to_string())
    );
    assert!(child2.metadata.get("prefix_label").is_some());
    assert!(child2.metadata.get("prefix_color").is_some());
}
