use bodo::config::{BodoConfig, TaskConfig};
use bodo::errors::BodoError;
use bodo::script_loader::ScriptLoader;
use std::collections::HashMap;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_duplicate_tasks_error() {
    // Construct a BodoConfig manually with duplicate tasks.
    let mut tasks = HashMap::new();
    let task_config = TaskConfig {
        command: Some("echo duplicate".to_string()),
        ..Default::default()
    };
    tasks.insert("dup".to_string(), task_config.clone());
    // Insert the same key again to simulate duplication.
    tasks.insert("dup".to_string(), task_config);
    let config = BodoConfig {
        tasks,
        ..Default::default()
    };
    let mut loader = ScriptLoader::new();
    let result = loader.build_graph(config);
    // Expect error for duplicate task.
    assert!(result.is_err());
    if let Err(BodoError::ValidationError(msg)) = result {
        assert!(msg.contains("duplicate task"));
    }
}

#[test]
fn test_invalid_dependency_error_in_concurrent_plugin() {
    // Build a graph with a Task node that has invalid concurrently metadata.
    use bodo::graph::{Graph, NodeKind, TaskData};
    let mut graph = Graph::new();
    let task_data = TaskData {
        name: "main".to_string(),
        description: None,
        command: Some("echo main".to_string()),
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
    };
    let main_id = graph.add_node(NodeKind::Task(task_data));
    // Set invalid concurrently metadata (non-JSON string).
    graph.nodes[main_id as usize]
        .metadata
        .insert("concurrently".to_string(), "not a json".to_string());
    use bodo::plugins::concurrent_plugin::ConcurrentPlugin;
    let mut plugin = ConcurrentPlugin::new();
    let result = plugin.on_graph_build(&mut graph);
    assert!(result.is_err());
    if let Err(BodoError::PluginError(msg)) = result {
        assert!(msg.contains("Invalid concurrency JSON"));
    }
}

#[test]
fn test_invalid_yaml_error() {
    // Write invalid YAML to a temporary file.
    let mut temp_file = NamedTempFile::new().unwrap();
    let invalid_yaml = "tasks:\n  task1:\n    command: echo 'Hello";
    fs::write(temp_file.path(), invalid_yaml).unwrap();
    let content = fs::read_to_string(temp_file.path()).unwrap();
    let result: Result<BodoConfig, _> = serde_yaml::from_str(&content);
    assert!(result.is_err());
}

#[test]
fn test_reserved_task_name_error() {
    // Create a BodoConfig with a reserved task name "watch".
    let mut tasks = HashMap::new();
    let task_config = TaskConfig {
        command: Some("echo reserved".to_string()),
        ..Default::default()
    };
    tasks.insert("watch".to_string(), task_config);
    let config = BodoConfig {
        tasks,
        ..Default::default()
    };
    let mut loader = ScriptLoader::new();
    let result = loader.build_graph(config);
    assert!(result.is_err());
    if let Err(BodoError::ValidationError(msg)) = result {
        assert!(msg.contains("reserved"));
    }
}
