use bodo::graph::{ConcurrentGroupData, Graph, NodeKind, TaskData};
use bodo::plugins::prefix_plugin::PrefixPlugin;
use std::collections::HashMap;

#[test]
fn test_prefix_plugin_with_children_in_group() {
    let mut plugin = PrefixPlugin::new();
    let mut graph = Graph::new();

    // Create a ConcurrentGroup node with prefix_output metadata; add child_ids in group data.
    let group_node_id = graph.add_node(NodeKind::ConcurrentGroup(ConcurrentGroupData {
        child_nodes: vec![],
        fail_fast: true,
        max_concurrent: None,
        timeout_secs: None,
    }));
    graph.nodes[group_node_id as usize]
        .metadata
        .insert("prefix_output".to_string(), "true".to_string());

    // Create two child tasks.
    let task1_id = graph.add_node(NodeKind::Task(TaskData {
        name: "child1".to_string(),
        description: Some("Child 1".to_string()),
        command: Some("echo child1".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "group_script".to_string(),
        script_display_name: "group_script".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    }));
    let task2_id = graph.add_node(NodeKind::Task(TaskData {
        name: "child2".to_string(),
        description: Some("Child 2".to_string()),
        command: Some("echo child2".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "group_script".to_string(),
        script_display_name: "group_script".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    }));

    // Update the ConcurrentGroup node to include these child nodes.
    if let NodeKind::ConcurrentGroup(ref mut group_data) = graph.nodes[group_node_id as usize].kind
    {
        group_data.child_nodes = vec![task1_id, task2_id];
    }

    // Run the prefix plugin.
    let result = plugin.on_graph_build(&mut graph);
    assert!(result.is_ok());

    // Check that each child task has prefix metadata set.
    let child1 = &graph.nodes[task1_id as usize];
    let child2 = &graph.nodes[task2_id as usize];
    assert_eq!(
        child1.metadata.get("prefix_enabled"),
        Some(&"true".to_string())
    );
    assert!(child1.metadata.get("prefix_label").is_some());
    assert!(child1.metadata.get("prefix_color").is_some());
    assert_eq!(
        child2.metadata.get("prefix_enabled"),
        Some(&"true".to_string())
    );
    assert!(child2.metadata.get("prefix_label").is_some());
    assert!(child2.metadata.get("prefix_color").is_some());
}
