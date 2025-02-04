use bodo::config::BodoConfig;
use bodo::script_loader::ScriptLoader;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_load_script_with_duplicate_tasks() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    // YAML duplicate keys: the second definition will override the first.
    let script_content = r#"tasks:
  duplicate_task:
    command: echo "First definition"
  duplicate_task:
    command: echo "Second definition"
"#;
    fs::write(&script_path, script_content).unwrap();

    let mut loader = ScriptLoader::new();
    // Use a configuration with no root_script so that tasks use keys as task names.
    let config_yaml = format!("tasks: {}", fs::read_to_string(&script_path).unwrap());
    let config: BodoConfig = serde_yaml::from_str(&config_yaml).unwrap();
    let graph = loader.build_graph(config).unwrap();
    // Expect the task_registry to contain key "duplicate_task"
    assert!(
        graph.task_registry.contains_key("duplicate_task"),
        "Expected task 'duplicate_task' to be present"
    );
    // Check that the command is from the second definition.
    if let Some(&node_id) = graph.task_registry.get("duplicate_task") {
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
    let config_yaml = format!("tasks: {}", fs::read_to_string(&script_path).unwrap());
    let config: BodoConfig = serde_yaml::from_str(&config_yaml).unwrap();
    let graph = loader.build_graph(config).unwrap();
    // Since dependency resolution is not performed during build_graph,
    // the pre_deps field remains; we can check that task1 exists and its pre_deps is not empty.
    let task1_node = graph
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
    if let bodo::graph::NodeKind::Task(task) = &task1_node.kind {
        assert_eq!(task.pre_deps.len(), 1, "Expected one pre_dep in task1");
    }
}

#[test]
fn test_load_script_with_invalid_task_name_chars() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    // Task name contains invalid characters.
    let script_content = r#"tasks:
  "invalid/task.name":
    command: echo "Invalid task name"
"#;
    fs::write(&script_path, script_content).unwrap();

    let loader = ScriptLoader::new();
    let config_yaml = format!("tasks: {}", fs::read_to_string(&script_path).unwrap());
    let result = serde_yaml::from_str::<BodoConfig>(&config_yaml);
    // Expect an error during deserialization/validation.
    assert!(
        result.is_err(),
        "Expected error due to invalid task name characters"
    );
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

    let loader = ScriptLoader::new();
    let config_result =
        serde_yaml::from_str::<BodoConfig>(&fs::read_to_string(&script_path).unwrap());
    assert!(
        config_result.is_err(),
        "Expected error due to invalid YAML syntax"
    );
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

    let loader = ScriptLoader::new();
    let config_yaml = format!("tasks: {}", fs::read_to_string(&script_path).unwrap());
    let result = serde_yaml::from_str::<BodoConfig>(&config_yaml);
    assert!(result.is_err(), "Expected error due to reserved task name");
}
