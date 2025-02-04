use bodo::graph::{Graph, NodeKind, TaskData};
use std::collections::HashMap;

#[test]
fn test_add_node() {
    let mut graph = Graph::new();
    let node_id = graph.add_node(NodeKind::Task(TaskData {
        name: "test_task".to_string(),
        description: None,
        command: None,
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
    assert_eq!(node_id, 0);
    assert_eq!(graph.nodes.len(), 1);
}

#[test]
fn test_graph_add_nodes_and_edges() {
    let mut graph = Graph::new();

    let task_data1 = TaskData {
        name: "task1".to_string(),
        description: None,
        command: Some("echo task1".to_string()),
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
    };
    let node_id1 = graph.add_node(NodeKind::Task(task_data1));

    let task_data2 = TaskData {
        name: "task2".to_string(),
        description: None,
        command: Some("echo task2".to_string()),
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
    };
    let node_id2 = graph.add_node(NodeKind::Task(task_data2));

    graph.add_edge(node_id1, node_id2).unwrap();

    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 1);
    assert_eq!(graph.edges[0].from, node_id1);
    assert_eq!(graph.edges[0].to, node_id2);
}
