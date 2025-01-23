use std::collections::HashMap;

use bodo::{
    errors::BodoError,
    graph::{CommandData, Edge, Graph, NodeKind, TaskData},
    Result,
};

#[test]
fn test_add_node() -> Result<()> {
    let mut graph = Graph::new();

    let node_id = graph.add_node(NodeKind::Task(TaskData {
        name: "test".to_string(),
        description: Some("Test task".to_string()),
        command: Some("echo test".to_string()),
        working_dir: None,
        is_default: false,
        script_id: "test_script".to_string(),
        script_display_name: "Test".to_string(),
        env: HashMap::new(),
    }));

    assert_eq!(node_id, 0);
    assert_eq!(graph.nodes.len(), 1);

    Ok(())
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
fn test_add_edge() -> Result<()> {
    let mut graph = Graph::new();

    let node1 = graph.add_node(NodeKind::Task(TaskData {
        name: "test1".to_string(),
        description: Some("Test task 1".to_string()),
        command: Some("echo test1".to_string()),
        working_dir: None,
        is_default: false,
        script_id: "test_script".to_string(),
        script_display_name: "Test".to_string(),
        env: HashMap::new(),
    }));

    let node2 = graph.add_node(NodeKind::Task(TaskData {
        name: "test2".to_string(),
        description: Some("Test task 2".to_string()),
        command: Some("echo test2".to_string()),
        working_dir: None,
        is_default: false,
        script_id: "test_script".to_string(),
        script_display_name: "Test".to_string(),
        env: HashMap::new(),
    }));

    let _ = graph.add_edge(node1, node2);

    assert_eq!(graph.edges.len(), 1);
    assert!(graph.edges.iter().any(|e| e.from == node1 && e.to == node2));

    Ok(())
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
        script_id: "test_script".to_string(),
        script_display_name: "Test".to_string(),
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
fn test_task_data() -> Result<()> {
    let task = TaskData {
        name: "test".to_string(),
        description: Some("Test task".to_string()),
        command: Some("echo test".to_string()),
        working_dir: None,
        is_default: false,
        script_id: "test_script".to_string(),
        script_display_name: "Test".to_string(),
        env: HashMap::new(),
    };

    assert_eq!(task.name, "test");
    assert_eq!(task.description, Some("Test task".to_string()));
    assert_eq!(task.command, Some("echo test".to_string()));
    assert_eq!(task.working_dir, None);
    assert_eq!(task.is_default, false);
    assert_eq!(task.env.len(), 0);

    Ok(())
}

#[test]
fn test_cycle_detection() -> Result<()> {
    let mut graph = Graph::new();

    let node1 = graph.add_node(NodeKind::Task(TaskData {
        name: "test1".to_string(),
        description: None,
        command: None,
        working_dir: None,
        is_default: false,
        script_id: "test_script".to_string(),
        script_display_name: "Test".to_string(),
        env: HashMap::new(),
    }));

    let node2 = graph.add_node(NodeKind::Task(TaskData {
        name: "test2".to_string(),
        description: None,
        command: None,
        working_dir: None,
        is_default: false,
        script_id: "test_script".to_string(),
        script_display_name: "Test".to_string(),
        env: HashMap::new(),
    }));

    let _ = graph.add_edge(node1, node2);
    let _ = graph.add_edge(node2, node1);

    assert!(graph.has_cycle());

    Ok(())
}
