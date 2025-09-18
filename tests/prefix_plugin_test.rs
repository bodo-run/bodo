use bodo::graph::{ConcurrentGroupData, Graph, NodeKind, TaskData};
use bodo::plugin::Plugin;
use bodo::plugins::prefix_plugin::PrefixPlugin;
use std::collections::HashMap;

#[test]
fn test_prefix_plugin_on_graph_build() {
    let mut plugin = PrefixPlugin::new();
    let mut graph = Graph::new();

    // Create a concurrent group node with prefix_output metadata set to "true"
    let group_node_id = graph.add_node(NodeKind::ConcurrentGroup(ConcurrentGroupData {
        child_nodes: vec![],
        fail_fast: true,
        max_concurrent: None,
        timeout_secs: None,
    }));
    {
        // Set prefix_output metadata for the group node
        let group_node = &mut graph.nodes[group_node_id as usize];
        group_node
            .metadata
            .insert("prefix_output".to_string(), "true".to_string());
    }
    // Create two child task nodes
    let task1_id = graph.add_node(NodeKind::Task(TaskData {
        name: "task1".to_string(),
        description: Some("Task one".to_string()),
        command: Some("echo Task1".to_string()),
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
    let task2_id = graph.add_node(NodeKind::Task(TaskData {
        name: "task2".to_string(),
        description: Some("Task two".to_string()),
        command: Some("echo Task2".to_string()),
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
    // Now, update the concurrent group's child_nodes vector to include these tasks.
    {
        if let NodeKind::ConcurrentGroup(ref mut group_data) =
            graph.nodes[group_node_id as usize].kind
        {
            group_data.child_nodes.push(task1_id);
            group_data.child_nodes.push(task2_id);
        }
    }
    // Call the plugin to update child nodes metadata.
    let result = plugin.on_graph_build(&mut graph);
    assert!(result.is_ok());
    // Check that each child task now has metadata for prefix_enabled, prefix_label, and prefix_color.
    for &child_id in &[task1_id, task2_id] {
        let child_node = &graph.nodes[child_id as usize];
        assert_eq!(
            child_node.metadata.get("prefix_enabled"),
            Some(&"true".to_string()),
            "Expected prefix_enabled to be \"true\""
        );
        assert!(
            child_node.metadata.contains_key("prefix_label"),
            "Expected prefix_label to be set"
        );
        assert!(
            child_node.metadata.contains_key("prefix_color"),
            "Expected prefix_color to be set"
        );
    }
}
