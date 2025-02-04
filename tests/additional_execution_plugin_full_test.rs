use bodo::{
    errors::{BodoError, Result},
    graph::{Graph, NodeKind, TaskData},
    plugin::PluginConfig,
    plugins::execution_plugin::ExecutionPlugin,
};
use std::collections::HashMap;

#[test]
fn test_execution_plugin_run_node_chain() -> Result<()> {
    // Create a graph with two tasks (B -> A). Task A depends on B.
    let mut graph = Graph::new();

    // Task B with a simple command.
    let task_b = TaskData {
        name: "B".to_string(),
        description: Some("Task B".to_string()),
        command: Some("echo B".to_string()),
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
    let b_id = graph.add_node(NodeKind::Task(task_b));

    // Task A with a simple command.
    let task_a = TaskData {
        name: "A".to_string(),
        description: Some("Task A".to_string()),
        command: Some("echo A".to_string()),
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
    let a_id = graph.add_node(NodeKind::Task(task_a));

    // Register tasks into the task_registry.
    graph.task_registry.insert("A".to_string(), a_id);
    graph.task_registry.insert("B".to_string(), b_id);

    // Add edge: Task B is a prerequisite for Task A.
    graph.add_edge(b_id, a_id).unwrap();

    // Create an ExecutionPlugin and set task_name to "A".
    let mut plugin = ExecutionPlugin::new();
    plugin.task_name = Some("A".to_string());
    let config = PluginConfig {
        fail_fast: true,
        watch: false,
        list: false,
        options: Some(
            serde_json::json!({"task": "A"})
                .as_object()
                .unwrap()
                .clone(),
        ),
    };
    plugin.on_init(&config).unwrap();

    // Call on_after_run which will recursively run dependencies.
    // The spawned commands ("echo A" and "echo B") will execute and complete.
    plugin
        .on_after_run(&mut graph)
        .map_err(|e| BodoError::PluginError(format!("Execution failed: {}", e)))
}
