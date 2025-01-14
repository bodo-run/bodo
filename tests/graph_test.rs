use bodo::graph::{CommandData, Graph, NodeKind, TaskData};

#[test]
fn test_empty_graph() {
    let graph = Graph::new();
    assert_eq!(graph.nodes.len(), 0);
    assert_eq!(graph.edges.len(), 0);
}

#[test]
fn test_add_single_task_node() {
    let mut graph = Graph::new();
    let task = TaskData {
        name: "build".to_string(),
        description: Some("Build the project".to_string()),
    };
    let node_id = graph.add_node(NodeKind::Task(task));

    // Check basic conditions
    assert_eq!(node_id, 0);
    assert_eq!(graph.nodes.len(), 1);
    assert_eq!(graph.edges.len(), 0);

    let node = &graph.nodes[node_id as usize];
    match &node.kind {
        NodeKind::Task(td) => {
            assert_eq!(td.name, "build");
            assert_eq!(td.description.as_deref(), Some("Build the project"));
        }
        _ => panic!("Expected a Task node"),
    }

    // Metadata is empty by default
    assert!(node.metadata.is_empty());
}

#[test]
fn test_add_multiple_command_nodes() {
    let mut graph = Graph::new();

    // Add first command node
    let cmd1 = CommandData {
        raw_command: "echo Hello".to_string(),
        description: None,
    };
    let id1 = graph.add_node(NodeKind::Command(cmd1));

    // Add second command node
    let cmd2 = CommandData {
        raw_command: "echo World".to_string(),
        description: Some("Print world".to_string()),
    };
    let id2 = graph.add_node(NodeKind::Command(cmd2));

    assert_eq!(id1, 0);
    assert_eq!(id2, 1);
    assert_eq!(graph.nodes.len(), 2);

    let node0 = &graph.nodes[0];
    let node1 = &graph.nodes[1];

    match &node0.kind {
        NodeKind::Command(cd) => {
            assert_eq!(cd.raw_command, "echo Hello");
            assert_eq!(cd.description, None);
        }
        _ => panic!("Expected a Command node"),
    }
    match &node1.kind {
        NodeKind::Command(cd) => {
            assert_eq!(cd.raw_command, "echo World");
            assert_eq!(cd.description.as_deref(), Some("Print world"));
        }
        _ => panic!("Expected a Command node"),
    }
}

#[test]
fn test_add_edges() {
    let mut graph = Graph::new();

    // Create nodes
    let task = TaskData {
        name: "test".to_string(),
        description: None,
    };
    let t_id = graph.add_node(NodeKind::Task(task));

    let cmd = CommandData {
        raw_command: "cargo test".to_string(),
        description: None,
    };
    let c_id = graph.add_node(NodeKind::Command(cmd));

    // Add edges
    graph.add_edge(t_id, c_id);
    // Add another edge (arbitrary example)
    graph.add_edge(c_id, t_id);

    assert_eq!(graph.edges.len(), 2);

    assert_eq!(graph.edges[0].from, 0);
    assert_eq!(graph.edges[0].to, 1);
    assert_eq!(graph.edges[1].from, 1);
    assert_eq!(graph.edges[1].to, 0);
}

#[test]
fn test_print_debug_no_panic() {
    let mut graph = Graph::new();
    // Should not panic or fail even if the graph is empty
    graph.print_debug();

    let _ = graph.add_node(NodeKind::Task(TaskData {
        name: "something".to_string(),
        description: None,
    }));
    let _ = graph.add_node(NodeKind::Command(CommandData {
        raw_command: "echo Testing".to_string(),
        description: None,
    }));
    graph.print_debug();
}
