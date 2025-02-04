use bodo::config::{BodoConfig, TaskConfig};
use bodo::errors::BodoError;
use bodo::plugin::Plugin;
use bodo::script_loader::ScriptLoader;
use std::fs;
use tempfile::tempdir; // Added to bring Plugin trait into scope

#[test]
fn test_load_script_with_duplicate_tasks() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    // Note: YAML duplicate keys are overwritten, so the second definition wins.
    let script_content = r#"tasks:
  duplicate_task:
    command: echo "First definition"
  duplicate_task:
    command: echo "Second definition"
"#;

    fs::write(&script_path, script_content).unwrap();

    let mut loader = ScriptLoader::new();
    let mut config = BodoConfig::default();
    config.scripts_dirs = Some(vec![temp_dir.path().to_string_lossy().to_string()]);

    // Expect the build_graph to succeed and keep the later definition.
    let graph = loader.build_graph(config).unwrap();
    let script_id = script_path.to_str().unwrap();
    let full_key = format!("{} {}", script_id, "duplicate_task");
    // Since duplicate keys are overwritten by YAML parsing,
    // the task registry should contain one entry with the final value.
    assert!(
        graph.task_registry.contains_key(&full_key),
        "Expected task '{}' to be present",
        full_key
    );
    // Optionally, one could further check that the task command equals the second definition.
    if let Some(&node_id) = graph.task_registry.get(&full_key) {
        if let bodo::graph::NodeKind::Task(task_data) = &graph.nodes[node_id as usize].kind {
            assert_eq!(
                task_data.command.as_deref(),
                Some("echo \"Second definition\"")
            );
        } else {
            panic!("Expected a Task node");
        }
    }
}

#[test]
fn test_load_script_with_invalid_dependency() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    let script_content = r#"tasks:
  task1:
    command: echo "Task 1"
    pre_deps:
      - task: non_existent_task
"#;

    fs::write(&script_path, script_content).unwrap();

    let mut loader = ScriptLoader::new();
    let mut config = BodoConfig::default();
    config.scripts_dirs = Some(vec![temp_dir.path().to_string_lossy().to_string()]);

    // Since dependency resolution is not performed in build_graph,
    // we expect build_graph to succeed and the pre_deps field to be preserved.
    let graph = loader.build_graph(config).unwrap();
    let task_node = graph
        .nodes
        .iter()
        .find(|node| {
            if let bodo::graph::NodeKind::Task(task) = &node.kind {
                task.name == "task1"
            } else {
                false
            }
        })
        .expect("Task 'task1' not found");
    if let bodo::graph::NodeKind::Task(task) = &task_node.kind {
        assert_eq!(task.pre_deps.len(), 1, "Expected one pre_dep");
    }
}

#[test]
fn test_load_script_with_invalid_yaml() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    // Invalid YAML: missing closing bracket.
    let script_content = r#"tasks:
  task1:
    command: echo "Task 1"
    pre_deps: [task2
"#;
    fs::write(&script_path, script_content).unwrap();

    let mut loader = ScriptLoader::new();
    let mut config = BodoConfig::default();
    config.scripts_dirs = Some(vec![temp_dir.path().to_string_lossy().to_string()]);

    let result = loader.build_graph(config);
    assert!(
        result.is_err(),
        "Expected YamlError due to invalid YAML syntax"
    );
    if let Err(BodoError::YamlError(_)) = result {
        // expected
    } else {
        panic!("Expected YamlError");
    }
}

#[test]
fn test_load_script_with_invalid_task_name_chars() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    // Task name contains '/' and '.'
    let script_content = r#"tasks:
  "invalid/task.name":
    command: echo "Invalid task name"
"#;
    fs::write(&script_path, script_content).unwrap();

    let mut loader = ScriptLoader::new();
    let mut config = BodoConfig::default();
    config.scripts_dirs = Some(vec![temp_dir.path().to_string_lossy().to_string()]);

    let result = loader.build_graph(config);
    assert!(
        result.is_err(),
        "Expected ValidationError due to invalid characters in task name"
    );
    if let Err(BodoError::ValidationError(_)) = result {
        // expected
    } else {
        panic!("Expected ValidationError for invalid task name");
    }
}

#[test]
fn test_load_script_with_reserved_task_name() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    // Using reserved task name "watch"
    let script_content = r#"tasks:
  watch:
    command: echo "This should fail"
"#;
    fs::write(&script_path, script_content).unwrap();

    let mut loader = ScriptLoader::new();
    let mut config = BodoConfig::default();
    config.scripts_dirs = Some(vec![temp_dir.path().to_string_lossy().to_string()]);

    let result = loader.build_graph(config);
    assert!(
        result.is_err(),
        "Expected ValidationError due to reserved task name"
    );
    if let Err(BodoError::ValidationError(_)) = result {
        // expected
    } else {
        panic!("Expected ValidationError for reserved task name");
    }
}
