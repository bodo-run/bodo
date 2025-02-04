use bodo::errors::BodoError;
use bodo::graph::{Graph, NodeKind, TaskData};

#[test]
fn test_add_edge_invalid() {
    let mut graph = Graph::new();
    // Create one task node.
    let _node0 = graph.add_node(NodeKind::Task(TaskData {
        name: "task".to_string(),
        description: None,
        command: Some("echo task".to_string()),
        working_dir: None,
        env: Default::default(),
        exec_paths: Vec::new(),
        arguments: Vec::new(),
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    }));
    // Attempt to add an edge from node 0 to node 1 (which doesn't exist).
    let err = graph.add_edge(0, 1);
    assert!(err.is_err());
    if let Err(BodoError::PluginError(msg)) = err {
        assert!(msg.contains("Invalid node ID"));
    } else {
        panic!("Expected PluginError for invalid edge");
    }
}

#[test]
fn test_get_node_name() {
    let mut graph = Graph::new();
    // Task node with empty script_display_name.
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "mytask".to_string(),
        description: None,
        command: Some("echo".to_string()),
        working_dir: None,
        env: Default::default(),
        exec_paths: Vec::new(),
        arguments: Vec::new(),
        is_default: false,
        script_id: "script1".to_string(),
        script_display_name: "".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    }));
    let name = graph.node_name(task_id as usize);
    assert_eq!(name, "mytask");

    // Task node with non-empty script_display_name.
    let task_id2 = graph.add_node(NodeKind::Task(TaskData {
        name: "task2".to_string(),
        description: None,
        command: Some("echo".to_string()),
        working_dir: None,
        env: Default::default(),
        exec_paths: Vec::new(),
        arguments: Vec::new(),
        is_default: false,
        script_id: "script2".to_string(),
        script_display_name: "scriptDir".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    }));
    let name2 = graph.node_name(task_id2 as usize);
    // When script_display_name is non-empty, expect the name to be combined.
    assert!(name2.contains("scriptDir") && name2.contains("task2"));
}
