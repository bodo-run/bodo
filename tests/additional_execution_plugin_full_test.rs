use bodo::errors::BodoError;
use bodo::graph::{Graph, NodeKind, TaskData};
use bodo::plugin::{Plugin, PluginConfig};
use bodo::plugins::execution_plugin::ExecutionPlugin;
use bodo::Result;
use serde_json::json;
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
        options: Some(json!({"task": "A"}).as_object().unwrap().clone()),
    };
    // Import Plugin trait for method resolution.
    plugin.on_init(&config).unwrap();

    // Call on_after_run which will recursively run dependencies.
    // The spawned commands ("echo A" and "echo B") will execute and complete.
    plugin
        .on_after_run(&mut graph)
        .map_err(|e| BodoError::PluginError(format!("Execution failed: {}", e)))
}

#[test]
fn test_execution_plugin_on_init() -> Result<()> {
    let mut plugin = ExecutionPlugin::new();
    let mut options = serde_json::Map::new();
    options.insert(
        "task".to_string(),
        serde_json::Value::String("example_task".to_string()),
    );
    let config = PluginConfig {
        fail_fast: true,
        watch: false,
        list: false,
        options: Some(options),
    };
    plugin.on_init(&config).unwrap();
    assert_eq!(plugin.task_name.as_deref(), Some("example_task"));
    Ok(())
}

#[test]
fn test_execution_plugin_on_after_run_no_task_specified() -> Result<()> {
    let mut plugin = ExecutionPlugin::new();
    let mut graph = Graph::new();
    let result = plugin.on_after_run(&mut graph);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_execution_plugin_on_after_run_with_command_node() -> Result<()> {
    let mut plugin = ExecutionPlugin::new();
    plugin.task_name = Some("test_task".to_string());
    let mut graph = Graph::new();
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "test_task".to_string(),
        description: None,
        command: Some("echo 'Hello World'".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: true,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    }));
    graph.task_registry.insert("test_task".to_string(), task_id);
    let result = plugin.on_after_run(&mut graph);
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_expand_env_vars_basic() {
    let env_map = HashMap::from([
        ("VAR1".to_string(), "value1".to_string()),
        ("VAR2".to_string(), "value2".to_string()),
    ]);
    let input = "echo $VAR1 and $VAR2";
    let expected = "echo value1 and value2";
    let plugin = ExecutionPlugin::new();
    let result = plugin.expand_env_vars(input, &env_map);
    assert_eq!(result, expected);
}

#[test]
fn test_expand_env_vars_no_match() {
    let env_map = HashMap::from([("VAR1".to_string(), "value1".to_string())]);
    let input = "echo $VAR2 and ${VAR3}";
    let expected = "echo $VAR2 and ${VAR3}";
    let plugin = ExecutionPlugin::new();
    let result = plugin.expand_env_vars(input, &env_map);
    assert_eq!(result, expected);
}

#[test]
fn test_expand_env_vars_partial() {
    let env_map = HashMap::from([("HOME".to_string(), "/home/user".to_string())]);
    let input = "cd $HOME/projects";
    let expected = "cd /home/user/projects";
    let plugin = ExecutionPlugin::new();
    let result = plugin.expand_env_vars(input, &env_map);
    assert_eq!(result, expected);
}

#[test]
fn test_expand_env_vars_special_chars() {
    let env_map = HashMap::from([("VAR".to_string(), "value".to_string())]);
    let input = "echo $$VAR $VAR$ $VAR text";
    let expected = "echo $VAR value$ value text";
    let plugin = ExecutionPlugin::new();
    let result = plugin.expand_env_vars(input, &env_map);
    assert_eq!(result, expected);
}

#[test]
fn test_expand_env_vars_empty_var() {
    let env_map = HashMap::new();
    let input = "echo $";
    let expected = "echo $";
    let plugin = ExecutionPlugin::new();
    let result = plugin.expand_env_vars(input, &env_map);
    assert_eq!(result, expected);
}

#[test]
fn test_execution_plugin_task_not_found() {
    let mut plugin = ExecutionPlugin::new();
    plugin.task_name = Some("nonexistent_task".to_string());
    let mut graph = Graph::new();
    let result = plugin.on_after_run(&mut graph);
    assert!(result.is_err());
}
