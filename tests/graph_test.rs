use std::collections::HashMap;

use bodo::{
    errors::BodoError,
    graph::{CommandData, Graph, NodeKind, TaskData},
};

#[test]
fn test_add_node() {
    let mut graph = Graph::new();

    let task = TaskData {
        name: "test".to_string(),
        description: Some("Test task".to_string()),
        command: Some("echo test".to_string()),
        working_dir: None,
        is_default: false,
        script_name: Some("Test".to_string()),
        env: HashMap::new(),
    };

    let task_id = graph.add_node(NodeKind::Task(task));
    assert_eq!(task_id, 0);
    assert_eq!(graph.nodes.len(), 1);
}

#[test]
fn test_add_command() {
    let mut graph = Graph::new();

    let command = CommandData {
        raw_command: "echo test".to_string(),
        description: None,
        working_dir: None,
        watch: None,
        env: HashMap::new(),
    };

    let cmd_id = graph.add_node(NodeKind::Command(command));
    assert_eq!(cmd_id, 0);
    assert_eq!(graph.nodes.len(), 1);
}

#[test]
fn test_add_edge() {
    let mut graph = Graph::new();

    let task = TaskData {
        name: "test".to_string(),
        description: Some("Test task".to_string()),
        command: Some("echo test".to_string()),
        working_dir: None,
        is_default: false,
        script_name: Some("Test".to_string()),
        env: HashMap::new(),
    };

    let command = CommandData {
        raw_command: "echo test".to_string(),
        description: None,
        working_dir: None,
        watch: None,
        env: HashMap::new(),
    };

    let task_id = graph.add_node(NodeKind::Task(task));
    let cmd_id = graph.add_node(NodeKind::Command(command));

    assert!(graph.add_edge(task_id, cmd_id).is_ok());
    assert_eq!(graph.edges.len(), 1);
}

#[test]
fn test_add_edge_invalid_node() {
    let mut graph = Graph::new();

    let task = TaskData {
        name: "test".to_string(),
        description: Some("Test task".to_string()),
        command: Some("echo test".to_string()),
        working_dir: None,
        is_default: false,
        script_name: Some("Test".to_string()),
        env: HashMap::new(),
    };

    let command = CommandData {
        raw_command: "echo test".to_string(),
        description: None,
        working_dir: None,
        watch: None,
        env: HashMap::new(),
    };

    let task_id = graph.add_node(NodeKind::Task(task));
    let cmd_id = graph.add_node(NodeKind::Command(command));

    assert!(matches!(
        graph.add_edge(task_id, cmd_id + 1),
        Err(BodoError::PluginError(_))
    ));
}

#[test]
fn test_has_cycle() {
    let mut graph = Graph::new();

    let _ = graph.add_node(NodeKind::Task(TaskData {
        name: "test1".to_string(),
        description: None,
        command: None,
        working_dir: None,
        is_default: false,
        script_name: Some("Test".to_string()),
        env: HashMap::new(),
    }));

    graph.print_debug();

    let _ = graph.add_node(NodeKind::Task(TaskData {
        name: "test2".to_string(),
        description: None,
        command: None,
        working_dir: None,
        is_default: false,
        script_name: Some("Test".to_string()),
        env: HashMap::new(),
    }));

    let _ = graph.add_node(NodeKind::Command(CommandData {
        raw_command: "echo test".to_string(),
        description: None,
        working_dir: None,
        watch: None,
        env: HashMap::new(),
    }));

    graph.print_debug();

    assert!(!graph.has_cycle());
}
