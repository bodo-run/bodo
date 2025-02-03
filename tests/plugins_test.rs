use bodo::errors::BodoError;
use bodo::graph::{CommandData, Graph, NodeKind, TaskData};
use bodo::plugin::{Plugin, PluginConfig};
use bodo::plugins::execution_plugin::ExecutionPlugin;
use std::collections::HashMap;

#[test]
fn test_execution_plugin_on_init() -> Result<(), BodoError> {
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
fn test_execution_plugin_on_after_run_no_task_specified() -> Result<(), BodoError> {
    let mut plugin = ExecutionPlugin::new();
    let mut graph = Graph::new();
    let result = plugin.on_after_run(&mut graph);
    assert!(
        matches!(result, Err(BodoError::PluginError(msg)) if msg.contains("No task specified"))
    );
    Ok(())
}

#[test]
fn test_execution_plugin_on_after_run() -> Result<(), BodoError> {
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
fn test_execution_plugin_with_command_node() -> Result<(), BodoError> {
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
