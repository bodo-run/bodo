use bodo::graph::{CommandData, Graph, NodeKind, TaskData};
use bodo::plugins::concurrent_plugin::ConcurrentPlugin;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_concurrent_plugin_object_dependency_task() {
    let mut graph = Graph::new();
    // Create a main task with "concurrently" metadata set to a JSON object with "task" key.
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
    // Set concurrently metadata to [{"task": "test_task"}]
    graph.nodes[main_id as usize].metadata.insert(
        "concurrently".to_string(),
        json!([{"task": "test_task"}]).to_string(),
    );

    // Create the dependency task "test_task"
    let dep_task = TaskData {
        name: "test_task".to_string(),
        description: None,
        command: Some("echo test_task".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "dep_script".to_string(),
        script_display_name: "dep_script".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    };
    let dep_id = graph.add_node(NodeKind::Task(dep_task));
    graph.task_registry.insert("test_task".to_string(), dep_id);

    let mut plugin = ConcurrentPlugin::new();
    let res = plugin.on_graph_build(&mut graph);
    assert!(res.is_ok());

    // Verify that the concurrent group node includes the dependency task.
    let group_nodes: Vec<_> = graph
        .nodes
        .iter()
        .filter(|node| matches!(node.kind, NodeKind::ConcurrentGroup(_)))
        .collect();
    assert_eq!(group_nodes.len(), 1);
    if let NodeKind::ConcurrentGroup(ref group_data) = group_nodes[0].kind {
        assert!(group_data.child_nodes.contains(&dep_id));
    } else {
        panic!("Expected a concurrent group node");
    }
}

#[test]
fn test_concurrent_plugin_object_dependency_command() {
    let mut graph = Graph::new();
    // Create a main task with "concurrently" metadata set to a JSON object with "command" key.
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
    // Set concurrently metadata to [{"command": "echo object command"}]
    graph.nodes[main_id as usize].metadata.insert(
        "concurrently".to_string(),
        json!([{"command": "echo object command"}]).to_string(),
    );

    let mut plugin = ConcurrentPlugin::new();
    let res = plugin.on_graph_build(&mut graph);
    assert!(res.is_ok());

    // Verify that a Command node was created and included in the concurrent group.
    let group_nodes: Vec<_> = graph
        .nodes
        .iter()
        .filter(|node| matches!(node.kind, NodeKind::ConcurrentGroup(_)))
        .collect();
    assert_eq!(group_nodes.len(), 1);
    let group_data = if let NodeKind::ConcurrentGroup(ref group_data) = group_nodes[0].kind {
        group_data
    } else {
        panic!("Expected a concurrent group node");
    };
    let command_found = group_data.child_nodes.iter().any(|&child_id| {
        if let NodeKind::Command(ref cmd_data) = graph.nodes[child_id as usize].kind {
            cmd_data.raw_command == "echo object command"
        } else {
            false
        }
    });
    assert!(
        command_found,
        "Concurrent group should contain the command node"
    );
}
