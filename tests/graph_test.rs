use bodo::graph::{CommandData, Graph, NodeKind, TaskData};

#[test]
fn test_empty_graph() {
    let graph = Graph::new();
    assert_eq!(graph.nodes.len(), 0);
    assert_eq!(graph.edges.len(), 0);
}

#[test]
fn test_add_task_node() {
    let mut graph = Graph::new();
    let task = TaskData {
        name: "test".to_string(),
        description: Some("test description".to_string()),
        command: None,
        working_dir: None,
    };
    let node_id = graph.add_node(NodeKind::Task(task.clone()));
    assert_eq!(node_id, 0);
    assert_eq!(graph.nodes.len(), 1);
    match &graph.nodes[0].kind {
        NodeKind::Task(t) => assert_eq!(t, &task),
        _ => panic!("Expected Task node"),
    }
}

#[test]
fn test_add_command_node() {
    let mut graph = Graph::new();
    let command = CommandData {
        raw_command: "echo hello".to_string(),
        description: Some("test description".to_string()),
        working_dir: None,
    };
    let node_id = graph.add_node(NodeKind::Command(command.clone()));
    assert_eq!(node_id, 0);
    assert_eq!(graph.nodes.len(), 1);
    match &graph.nodes[0].kind {
        NodeKind::Command(c) => assert_eq!(c, &command),
        _ => panic!("Expected Command node"),
    }
}

#[test]
fn test_add_multiple_nodes() {
    let mut graph = Graph::new();
    let task = TaskData {
        name: "test".to_string(),
        description: Some("test description".to_string()),
        command: None,
        working_dir: None,
    };
    let command = CommandData {
        raw_command: "echo hello".to_string(),
        description: Some("test description".to_string()),
        working_dir: None,
    };
    let task_id = graph.add_node(NodeKind::Task(task.clone()));
    let command_id = graph.add_node(NodeKind::Command(command.clone()));
    assert_eq!(task_id, 0);
    assert_eq!(command_id, 1);
    assert_eq!(graph.nodes.len(), 2);
    match &graph.nodes[0].kind {
        NodeKind::Task(t) => assert_eq!(t, &task),
        _ => panic!("Expected Task node"),
    }
    match &graph.nodes[1].kind {
        NodeKind::Command(c) => assert_eq!(c, &command),
        _ => panic!("Expected Command node"),
    }
}

#[test]
fn test_add_edge() {
    let mut graph = Graph::new();
    let task = TaskData {
        name: "test".to_string(),
        description: Some("test description".to_string()),
        command: None,
        working_dir: None,
    };
    let command = CommandData {
        raw_command: "echo hello".to_string(),
        description: Some("test description".to_string()),
        working_dir: None,
    };
    let task_id = graph.add_node(NodeKind::Task(task));
    let command_id = graph.add_node(NodeKind::Command(command));
    let _ = graph.add_edge(task_id, command_id);
    assert_eq!(graph.edges.len(), 1);
    assert_eq!(graph.edges[0].from, task_id);
    assert_eq!(graph.edges[0].to, command_id);
}

#[test]
fn test_add_edge_invalid_nodes() {
    let mut graph = Graph::new();
    let _ = graph.add_node(NodeKind::Task(TaskData {
        name: "test".to_string(),
        description: Some("test description".to_string()),
        command: None,
        working_dir: None,
    }));
    assert!(graph.add_edge(0, 1).is_err());
    assert!(graph.add_edge(1, 0).is_err());
}

#[test]
fn test_print_debug_no_panic() {
    let mut graph = Graph::new();
    // Should not panic or fail even if the graph is empty
    graph.print_debug();

    let _ = graph.add_node(NodeKind::Task(TaskData {
        name: "something".to_string(),
        description: None,
        command: None,
        working_dir: None,
    }));
    let _ = graph.add_node(NodeKind::Command(CommandData {
        raw_command: "echo Testing".to_string(),
        description: None,
        working_dir: None,
    }));
    graph.print_debug();
}
