use bodo::errors::{BodoError, Result};
use bodo::graph::{CommandData, Graph, NodeKind, TaskData};
use bodo::plugin::{Plugin, PluginConfig};
use bodo::plugins::execution_plugin::{expand_env_vars, ExecutionPlugin};
use std::collections::HashMap;

#[test]
fn test_execution_plugin_on_init() -> Result<()> {
    let mut plugin = ExecutionPlugin::new();
    let mut options = serde_json::Map::new();
    options.insert(
        "task".to_string(),
        serde_json::Value::String("test_task".to_string()),
    );
    let config = PluginConfig {
        fail_fast: true,
        watch: false,
        list: false,
        options: Some(options),
    };
    plugin.on_init(&config)?;
    assert_eq!(plugin.task_name.as_deref(), Some("test_task"));
    Ok(())
}

#[test]
fn test_execution_plugin_on_after_run_no_task_specified() -> Result<()> {
    let mut plugin = ExecutionPlugin::new();
    let mut graph = Graph::new();
    let result = plugin.on_after_run(&mut graph);
    assert!(matches!(result, Err(BodoError::PluginError(_))));
    Ok(())
}

#[test]
fn test_execution_plugin_on_after_run() -> Result<()> {
    let mut plugin = ExecutionPlugin::new();
    plugin.task_name = Some("test_task".to_string());
    let mut graph = Graph::new();
    let node_id = graph.add_node(NodeKind::Task(TaskData {
        name: "test_task".to_string(),
        description: None,
        command: Some("echo 'Hello World'".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: true,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
    }));
    graph.task_registry.insert("test_task".to_string(), node_id);
    let result = plugin.on_after_run(&mut graph);
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_execution_plugin_with_command_node() -> Result<()> {
    let mut plugin = ExecutionPlugin::new();
    plugin.task_name = Some("test_task".to_string());
    let mut graph = Graph::new();
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "test_task".to_string(),
        description: None,
        command: None,
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: true,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
    }));
    graph.task_registry.insert("test_task".to_string(), task_id);
    let command_id = graph.add_node(NodeKind::Command(CommandData {
        raw_command: "echo 'Command Node'".to_string(),
        description: None,
        working_dir: None,
        env: HashMap::new(),
        watch: None,
    }));
    graph.add_edge(task_id, command_id)?;
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
    let result = expand_env_vars(input, &env_map);
    assert_eq!(result, expected);
}

#[test]
fn test_expand_env_vars_no_match() {
    let env_map = HashMap::from([("VAR1".to_string(), "value1".to_string())]);
    let input = "echo $VAR2 and ${VAR3}";
    let expected = "echo $VAR2 and ${VAR3}";
    let result = expand_env_vars(input, &env_map);
    assert_eq!(result, expected);
}

#[test]
fn test_expand_env_vars_partial() {
    let env_map = HashMap::from([("HOME".to_string(), "/home/user".to_string())]);
    let input = "cd $HOME/projects";
    let expected = "cd /home/user/projects";
    let result = expand_env_vars(input, &env_map);
    assert_eq!(result, expected);
}

#[test]
fn test_expand_env_vars_special_chars() {
    let env_map = HashMap::from([("VAR".to_string(), "value".to_string())]);
    let input = "echo $$VAR $VAR$ $VAR text";
    let expected = "echo $VAR value$ value text";
    let result = expand_env_vars(input, &env_map);
    assert_eq!(result, expected);
}

#[test]
fn test_expand_env_vars_empty_var() {
    let env_map = HashMap::new();
    let input = "echo $";
    let expected = "echo $";
    let result = expand_env_vars(input, &env_map);
    assert_eq!(result, expected);
}
