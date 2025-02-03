// tests/script_loader_error_test.rs

use bodo::config::BodoConfig;
use bodo::errors::BodoError;
use bodo::graph::NodeKind;
use bodo::script_loader::ScriptLoader;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_load_script_with_duplicate_tasks() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    // Due to serde_yaml limitations, duplicate keys are not treated as an error, and the last occurrence overwrites the previous one
    // So we cannot expect an error in this case
    // Instead, we can check that the task named 'duplicate_task' has the command from the second definition

    let script_content = r#"
tasks:
  duplicate_task:
    command: echo "First definition"
  duplicate_task:
    command: echo "Second definition"
"#;

    fs::write(&script_path, script_content).unwrap();

    let mut loader = ScriptLoader::new();
    let mut config = BodoConfig::default();
    config.scripts_dirs = Some(vec![temp_dir.path().to_string_lossy().to_string()]);

    let result = loader.build_graph(config);
    assert!(
        result.is_ok(),
        "Expected build_graph to succeed, but it failed: {:?}",
        result
    );

    let graph = result.unwrap();

    // Now, we can check that the task 'duplicate_task' exists in the graph, and its command is 'echo "Second definition"'

    let task_node_id = graph.task_registry.iter().find_map(|(k, v)| {
        if k.contains("duplicate_task") {
            Some(v)
        } else {
            None
        }
    });
    assert!(
        task_node_id.is_some(),
        "Expected to find 'duplicate_task' in task_registry"
    );

    let node_id = *task_node_id.unwrap();
    let node = &graph.nodes[node_id as usize];

    if let NodeKind::Task(task_data) = &node.kind {
        assert_eq!(
            task_data.name, "duplicate_task",
            "Expected task name to be 'duplicate_task', found '{}'",
            task_data.name
        );
        assert_eq!(
            task_data.command.as_deref(),
            Some("echo \"Second definition\""),
            "Expected command to be 'echo \"Second definition\"', found '{:?}'",
            task_data.command
        );
    } else {
        panic!("Expected node of kind Task");
    }
}

#[test]
fn test_load_script_with_reserved_task_name() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    let script_content = r#"
tasks:
  watch:
    command: echo "This should fail"
"#;

    fs::write(&script_path, script_content).unwrap();

    let mut loader = ScriptLoader::new();
    let mut config = BodoConfig::default();
    config.scripts_dirs = Some(vec![temp_dir.path().to_string_lossy().to_string()]);

    let result = loader.build_graph(config);
    assert!(
        matches!(result, Err(BodoError::ValidationError(_))),
        "Expected ValidationError due to reserved task name"
    );
}

#[test]
fn test_load_script_with_invalid_dependency() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    let script_content = r#"
tasks:
  task1:
    command: echo "Task 1"
    pre_deps:
      - task: non_existent_task
"#;

    fs::write(&script_path, script_content).unwrap();

    let mut loader = ScriptLoader::new();
    let mut config = BodoConfig::default();
    config.scripts_dirs = Some(vec![temp_dir.path().to_string_lossy().to_string()]);

    let result = loader.build_graph(config);
    assert!(
        matches!(result, Err(BodoError::PluginError(_))),
        "Expected PluginError due to invalid dependency"
    );
}

#[test]
fn test_load_script_with_invalid_task_name_chars() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    let script_content = r#"
tasks:
  invalid/task.name:
    command: echo "Invalid task name"
"#;

    fs::write(&script_path, script_content).unwrap();

    let mut loader = ScriptLoader::new();
    let mut config = BodoConfig::default();
    config.scripts_dirs = Some(vec![temp_dir.path().to_string_lossy().to_string()]);

    let result = loader.build_graph(config);
    assert!(
        matches!(result, Err(BodoError::ValidationError(_))),
        "Expected ValidationError due to invalid characters in task name"
    );
}

#[test]
fn test_load_script_with_invalid_yaml() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    let script_content = r#"
tasks:
  task1:
    command: echo "Task 1"
    pre_deps: [task2
"#; // Missing closing bracket

    fs::write(&script_path, script_content).unwrap();

    let mut loader = ScriptLoader::new();
    let mut config = BodoConfig::default();
    config.scripts_dirs = Some(vec![temp_dir.path().to_string_lossy().to_string()]);

    let result = loader.build_graph(config);
    assert!(
        matches!(result, Err(BodoError::YamlError(_))),
        "Expected YamlError due to invalid YAML syntax"
    );
}
