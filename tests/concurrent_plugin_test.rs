use bodo::graph::{Graph, NodeKind, TaskData};
use bodo::plugin::Plugin;
use bodo::plugins::concurrent_plugin::ConcurrentPlugin;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_concurrent_plugin() {
    let mut plugin = ConcurrentPlugin::new();

    let mut graph = Graph::new();

    // Create tasks
    let task_data_main = TaskData {
        name: "main_task".to_string(),
        description: None,
        command: None, // No command, will have concurrent tasks
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
        is_default: true,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    };

    let main_task_id = graph.add_node(NodeKind::Task(Box::new(task_data_main)));

    let task_data_child1 = TaskData {
        name: "child_task1".to_string(),
        description: None,
        command: Some("echo Child 1".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    };

    let child1_id = graph.add_node(NodeKind::Task(Box::new(task_data_child1)));

    let task_data_child2 = TaskData {
        name: "child_task2".to_string(),
        description: None,
        command: Some("echo Child 2".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    };

    let child2_id = graph.add_node(NodeKind::Task(Box::new(task_data_child2)));

    // Register child tasks in task_registry
    graph
        .task_registry
        .insert("child_task1".to_string(), child1_id);
    graph
        .task_registry
        .insert("child_task2".to_string(), child2_id);

    // Set up the main_task to have concurrent tasks
    let main_node = &mut graph.nodes[main_task_id as usize];
    // Set the metadata 'concurrently' directly as a JSON array string
    main_node.metadata.insert(
        "concurrently".to_string(),
        "[\"child_task1\", \"child_task2\"]".to_string(),
    );
    // Also set 'fail_fast' and 'max_concurrent' in metadata
    main_node
        .metadata
        .insert("fail_fast".to_string(), "true".to_string());
    main_node
        .metadata
        .insert("max_concurrent".to_string(), "2".to_string());

    // Apply the plugin
    let result = plugin.on_graph_build(&mut graph);
    assert!(
        result.is_ok(),
        "Plugin on_graph_build returned an error: {:?}",
        result.unwrap_err()
    );

    // Check that a ConcurrentGroup node has been added
    let group_nodes: Vec<_> = graph
        .nodes
        .iter()
        .filter_map(|node| {
            if let NodeKind::ConcurrentGroup(group_data) = &node.kind {
                Some((node.id, group_data))
            } else {
                None
            }
        })
        .collect();

    assert_eq!(group_nodes.len(), 1, "Expected one concurrent group node");

    let (group_id, group_data) = &group_nodes[0];
    assert_eq!(group_data.child_nodes.len(), 2);
    assert!(group_data.child_nodes.contains(&child1_id));
    assert!(group_data.child_nodes.contains(&child2_id));

    // Check the 'fail_fast' and 'max_concurrent' settings
    assert!(group_data.fail_fast, "Expected fail_fast to be true");
    assert_eq!(
        group_data.max_concurrent,
        Some(2),
        "Expected max_concurrent to be 2"
    );

    // Check that edges have been added appropriately
    // Edge from main_task to group
    assert!(graph
        .edges
        .iter()
        .any(|edge| edge.from == main_task_id && edge.to == *group_id));

    // Edges from group to child tasks
    assert!(graph
        .edges
        .iter()
        .any(|edge| edge.from == *group_id && edge.to == child1_id));
    assert!(graph
        .edges
        .iter()
        .any(|edge| edge.from == *group_id && edge.to == child2_id));
}

#[test]
fn test_concurrent_plugin_with_commands() {
    let mut plugin = ConcurrentPlugin::new();

    let mut graph = Graph::new();

    // Create a main task
    let task_data_main = TaskData {
        name: "main_task".to_string(),
        description: None,
        command: None, // No command, will have concurrent commands
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
        is_default: true,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    };

    let main_task_id = graph.add_node(NodeKind::Task(Box::new(task_data_main)));

    // Set up the main_task to have concurrent commands
    let main_node = &mut graph.nodes[main_task_id as usize];
    // Set the metadata 'concurrently' directly
    main_node.metadata.insert(
        "concurrently".to_string(),
        r#"[{"command": "echo Command 1"}, {"command": "echo Command 2"}]"#.to_string(),
    );

    // Apply the plugin
    let result = plugin.on_graph_build(&mut graph);
    assert!(
        result.is_ok(),
        "Plugin on_graph_build returned an error: {:?}",
        result.unwrap_err()
    );

    // Check that a ConcurrentGroup node has been added
    let group_nodes: Vec<_> = graph
        .nodes
        .iter()
        .filter_map(|node| {
            if let NodeKind::ConcurrentGroup(group_data) = &node.kind {
                Some((node.id, group_data))
            } else {
                None
            }
        })
        .collect();

    assert_eq!(group_nodes.len(), 1, "Expected one concurrent group node");

    let (group_id, group_data) = &group_nodes[0];
    assert_eq!(group_data.child_nodes.len(), 2);

    // The child nodes should be Command nodes
    for &child_id in &group_data.child_nodes {
        let child_node = &graph.nodes[child_id as usize];
        if let NodeKind::Command(cmd_data) = &child_node.kind {
            assert!(
                cmd_data.raw_command == "echo Command 1"
                    || cmd_data.raw_command == "echo Command 2"
            );
        } else {
            panic!("Expected Command node");
        }
    }
}

#[test]
fn test_concurrent_plugin_nonexistent_task() {
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
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
        is_default: true,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    };

    let main_task_id = graph.add_node(NodeKind::Task(Box::new(task_data_main)));

    // Set up the main_task to have a nonexistent concurrent task
    let main_node = &mut graph.nodes[main_task_id as usize];
    main_node.metadata.insert(
        "concurrently".to_string(),
        "[\"nonexistent_task\"]".to_string(),
    );

    // Apply the plugin
    let result = plugin.on_graph_build(&mut graph);
    assert!(
        result.is_err(),
        "Expected error due to nonexistent task, but got success"
    );
    let error = result.unwrap_err();
    assert!(
        matches!(error, bodo::errors::BodoError::PluginError(_)),
        "Expected PluginError, got {:?}",
        error
    );
}

#[test]
fn test_concurrent_plugin_invalid_dependency_format() {
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
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
        is_default: true,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    };

    let main_task_id = graph.add_node(NodeKind::Task(Box::new(task_data_main)));

    // Set up the main_task to have an invalid concurrent dependency
    let main_node = &mut graph.nodes[main_task_id as usize];
    main_node.metadata.insert(
        "concurrently".to_string(),
        "[123, true]".to_string(), // Invalid format
    );

    // Apply the plugin
    let result = plugin.on_graph_build(&mut graph);
    assert!(
        result.is_err(),
        "Expected error due to invalid dependency format, but got success"
    );
    let error = result.unwrap_err();
    assert!(
        matches!(error, bodo::errors::BodoError::PluginError(_)),
        "Expected PluginError, got {:?}",
        error
    );
}

#[test]
fn test_concurrent_plugin_empty_concurrently() {
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
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
        is_default: true,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    };

    let main_task_id = graph.add_node(NodeKind::Task(Box::new(task_data_main)));

    // Set up the main_task with empty 'concurrently' metadata
    let main_node = &mut graph.nodes[main_task_id as usize];
    main_node
        .metadata
        .insert("concurrently".to_string(), "[]".to_string());

    // Apply the plugin
    let result = plugin.on_graph_build(&mut graph);
    assert!(
        result.is_ok(),
        "Plugin on_graph_build returned an error: {:?}",
        result.unwrap_err()
    );

    // Check that a ConcurrentGroup node has been added with no children
    let group_nodes: Vec<_> = graph
        .nodes
        .iter()
        .filter_map(|node| {
            if let NodeKind::ConcurrentGroup(group_data) = &node.kind {
                Some((node.id, group_data))
            } else {
                None
            }
        })
        .collect();

    assert_eq!(group_nodes.len(), 1, "Expected one concurrent group node");

    let (_group_id, group_data) = &group_nodes[0];
    assert_eq!(
        group_data.child_nodes.len(),
        0,
        "Expected no child nodes in the group"
    );
}

#[test]
fn test_concurrent_plugin_nonexistent_task_in_object() {
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
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
        is_default: true,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    };
    let main_task_id = graph.add_node(NodeKind::Task(Box::new(task_data_main)));
    let main_node = &mut graph.nodes[main_task_id as usize];
    main_node.metadata.insert(
        "concurrently".to_string(),
        serde_json::to_string(&json!([{"task": "nonexistent"}])).unwrap(),
    );
    let result = plugin.on_graph_build(&mut graph);
    assert!(
        result.is_err(),
        "Expected error for nonexistent task in object dependency"
    );
}
