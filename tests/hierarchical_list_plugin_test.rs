use bodo::{
    graph::{Graph, NodeKind, TaskData},
    plugins::hierarchical_list_plugin::HierarchicalListPlugin,
};

#[test]
fn test_empty_graph() {
    let plugin = HierarchicalListPlugin::new();
    let graph = Graph::new();
    let output = plugin.build_hierarchical_list(&graph);
    assert_eq!(output, "No tasks found.\n");
}

#[test]
fn test_root_level_tasks() {
    let mut graph = Graph::new();
    let plugin = HierarchicalListPlugin::new();

    graph.add_node(NodeKind::Task(TaskData {
        name: "default_task".to_string(),
        description: Some("Default greeting".to_string()),
        command: None,
        working_dir: None,
    }));

    graph.add_node(NodeKind::Task(TaskData {
        name: "echo".to_string(),
        description: Some("echo task".to_string()),
        command: Some("echo Hello".to_string()),
        working_dir: None,
    }));

    let output = plugin.build_hierarchical_list(&graph);
    assert!(output.contains("Root level tasks"));
    assert!(output.contains("default_task"));
    assert!(output.contains("Default greeting"));
    assert!(output.contains("echo"));
    assert!(output.contains("echo task"));
}

#[test]
fn test_script_tasks() {
    let mut graph = Graph::new();
    let plugin = HierarchicalListPlugin::new();

    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "clippy".to_string(),
        description: Some("Run clippy".to_string()),
        command: Some("cargo clippy".to_string()),
        working_dir: None,
    }));

    graph.nodes[task_id as usize].metadata.insert(
        "script_source".to_string(),
        "scripts/code_quality.yaml".to_string(),
    );
    graph.nodes[task_id as usize].metadata.insert(
        "script_description".to_string(),
        "Code quality commands".to_string(),
    );

    let output = plugin.build_hierarchical_list(&graph);
    assert!(output.contains("scripts/code_quality.yaml"));
    assert!(output.contains("Code quality commands"));
    assert!(output.contains("clippy"));
    assert!(output.contains("Run clippy"));
}

#[test]
fn test_mixed_tasks() {
    let mut graph = Graph::new();
    let plugin = HierarchicalListPlugin::new();

    // Root level task
    graph.add_node(NodeKind::Task(TaskData {
        name: "root_task".to_string(),
        description: Some("Root task".to_string()),
        command: None,
        working_dir: None,
    }));

    // Script task
    let script_task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "script_task".to_string(),
        description: Some("Script task".to_string()),
        command: None,
        working_dir: None,
    }));

    graph.nodes[script_task_id as usize]
        .metadata
        .insert("script_source".to_string(), "scripts/test.yaml".to_string());

    let output = plugin.build_hierarchical_list(&graph);
    assert!(output.contains("Root level tasks"));
    assert!(output.contains("root_task"));
    assert!(output.contains("Root task"));
    assert!(output.contains("scripts/test.yaml"));
    assert!(output.contains("script_task"));
    assert!(output.contains("Script task"));
}
