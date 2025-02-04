use bodo::graph::{CommandData, ConcurrentGroupData, Graph, NodeKind, TaskData};
use std::collections::HashMap;

#[test]
fn test_print_debug_and_get_node_name() {
    let mut graph = Graph::new();

    // Add a Task node
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "TaskA".to_string(),
        description: Some("Task A Description".to_string()),
        command: Some("echo TaskA".to_string()),
        working_dir: Some("cwd".to_string()),
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: true,
        script_id: "script".to_string(),
        script_display_name: "scriptDir".to_string(),
        watch: None,
    }));
    let task_name = graph.get_node_name(task_id as usize);
    assert!(task_name.contains("TaskA") || task_name.contains("scriptDir/TaskA"));

    // Add a Command node
    let cmd_id = graph.add_node(NodeKind::Command(CommandData {
        raw_command: "echo Command".to_string(),
        description: Some("Command Description".to_string()),
        working_dir: None,
        env: HashMap::new(),
        watch: None,
    }));
    let cmd_name = graph.get_node_name(cmd_id as usize);
    assert!(cmd_name.contains("command"));

    // Add a ConcurrentGroup node
    let group_id = graph.add_node(NodeKind::ConcurrentGroup(ConcurrentGroupData {
        child_nodes: vec![task_id, cmd_id],
        fail_fast: false,
        max_concurrent: Some(2),
        timeout_secs: Some(10),
    }));
    let group_name = graph.get_node_name(group_id as usize);
    assert!(group_name.contains("concurrent_group"));

    // Call print_debug to boost coverage. (The output goes to log, so we just call the function.)
    graph.print_debug();
}

#[test]
fn test_get_node_name_for_various_types() {
    let mut graph = Graph::new();

    // Task node with empty script_display_name
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "task_only".to_string(),
        description: None,
        command: Some("echo test".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
    }));
    let name1 = graph.get_node_name(task_id as usize);
    assert_eq!(name1, "task_only");

    // Command node
    let cmd_id = graph.add_node(NodeKind::Command(CommandData {
        raw_command: "ls -la".to_string(),
        description: None,
        working_dir: None,
        env: HashMap::new(),
        watch: None,
    }));
    let name2 = graph.get_node_name(cmd_id as usize);
    assert!(name2.contains("command"));

    // ConcurrentGroup node
    let group_id = graph.add_node(NodeKind::ConcurrentGroup(ConcurrentGroupData {
        child_nodes: vec![],
        fail_fast: true,
        max_concurrent: None,
        timeout_secs: None,
    }));
    let name3 = graph.get_node_name(group_id as usize);
    assert!(name3.contains("concurrent_group"));
}
