use bodo::config::BodoConfig;
use bodo::script_loader::ScriptLoader;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_load_script_with_duplicate_tasks() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    // YAML with duplicate keys. Due to YAML spec the later key overwrites the earlier one.
    let script_content = r#"
tasks:
  duplicate_task:
    command: echo "First definition"
  duplicate_task:
    command: echo "Second definition"
"#;
    fs::write(&script_path, script_content).unwrap();
    let _loader = ScriptLoader::new();
    let file_content = fs::read_to_string(&script_path).unwrap();
    // Parsing will NOT error due to duplicates; the last value wins.
    let config = serde_yaml::from_str::<BodoConfig>(&file_content)
        .expect("Parsing should succeed even if duplicate keys occur");
    // Build graph using the config.
    let mut loader = ScriptLoader::new();
    let graph = loader
        .build_graph(config)
        .expect("Graph build should succeed");
    // When no root_script is provided, keys are inserted as task name.
    // Therefore, duplicate_task should be present only once.
    let count = graph
        .task_registry
        .keys()
        .filter(|k| k == &&"duplicate_task".to_string())
        .count();
    assert_eq!(
        count, 1,
        "Duplicate task key should result in one registered task"
    );
    // Optionally, you can check that the command of the task is the one from the second definition.
    let key = "duplicate_task".to_string();
    let node_id = graph.task_registry.get(&key).expect("Task not found");
    if let bodo::graph::NodeKind::Task(Box::new(task) = &graph.nodes[*node_id as usize].kind {
        assert_eq!(task.command.as_deref(), Some("echo \"Second definition\""));
    } else {
        panic!("Expected a Task node");
    }
}

#[test]
fn test_load_script_with_invalid_dependency_format() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    let script_content = r#"
tasks:
  task1:
    command: echo "Task 1"
    pre_deps:
      - 123
"#;
    fs::write(&script_path, script_content).unwrap();

    let file_content = fs::read_to_string(&script_path).unwrap();
    let res = serde_yaml::from_str::<BodoConfig>(&file_content);
    assert!(
        res.is_err(),
        "Expected error due to invalid dependency format"
    );
}
