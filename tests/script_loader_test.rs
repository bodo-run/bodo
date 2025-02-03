use bodo::config::BodoConfig;
use bodo::errors::BodoError;
use bodo::manager::GraphManager;
use bodo::script_loader::ScriptLoader;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_load_script() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    let script_content = r#"
    tasks:
      test_task:
        command: echo "Test Task"
    "#;

    fs::write(&script_path, script_content).unwrap();

    let mut loader = ScriptLoader::new();
    let mut config = BodoConfig::default();
    config.scripts_dirs = Some(vec![temp_dir.path().to_string_lossy().to_string()]);

    let graph = loader.build_graph(config).unwrap();
    assert!(graph.task_registry.contains_key("test_task"));
}

#[test]
fn test_load_scripts_dir() {
    let temp_dir = tempdir().unwrap();
    let scripts_dir = temp_dir.path().join("scripts");
    fs::create_dir(&scripts_dir).unwrap();

    let script1_path = scripts_dir.join("script1.yaml");
    let script2_path = scripts_dir.join("script2.yaml");

    let script1_content = r#"
    tasks:
      task1:
        command: echo "Task 1"
    "#;

    let script2_content = r#"
    tasks:
      task2:
        command: echo "Task 2"
    "#;

    fs::write(&script1_path, script1_content).unwrap();
    fs::write(&script2_path, script2_content).unwrap();

    let mut loader = ScriptLoader::new();
    let mut config = BodoConfig::default();
    config.scripts_dirs = Some(vec![scripts_dir.to_string_lossy().to_string()]);

    let graph = loader.build_graph(config).unwrap();
    assert!(graph.task_registry.contains_key("task1"));
    assert!(graph.task_registry.contains_key("task2"));
}

#[test]
fn test_task_dependencies() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    let script_content = r#"
    tasks:
      task1:
        command: echo "Task 1"
        pre_deps:
          - task: task2
      task2:
        command: echo "Task 2"
    "#;

    fs::write(&script_path, script_content).unwrap();

    let mut loader = ScriptLoader::new();
    let mut config = BodoConfig::default();
    config.scripts_dirs = Some(vec![temp_dir.path().to_string_lossy().to_string()]);

    let graph = loader.build_graph(config).unwrap();
    assert!(graph.task_registry.contains_key("task1"));
    assert!(graph.task_registry.contains_key("task2"));

    // Check that there's an edge from task2 to task1
    let task1_id = graph.task_registry.get("task1").unwrap();
    let task2_id = graph.task_registry.get("task2").unwrap();

    let mut found = false;
    for edge in &graph.edges {
        if edge.from == *task2_id && edge.to == *task1_id {
            found = true;
            break;
        }
    }
    assert!(found, "Edge from task2 to task1 not found");
}

#[test]
fn test_cycle_detection() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    let script_content = r#"
    tasks:
      task1:
        command: echo "Task 1"
        pre_deps:
          - task: task2
      task2:
        command: echo "Task 2"
        pre_deps:
          - task: task1
    "#;

    fs::write(&script_path, script_content).unwrap();

    let mut config = BodoConfig::default();
    config.scripts_dirs = Some(vec![temp_dir.path().to_string_lossy().to_string()]);

    // This should result in a cycle in the graph
    let mut manager = GraphManager::new();
    let result = manager.build_graph(config);

    assert!(result.is_err(), "Cycle was not detected");
    match result {
        Err(BodoError::PluginError(msg)) => {
            assert!(
                msg.contains("found cyclical dependency"),
                "Incorrect error message: {}",
                msg
            );
        }
        _ => panic!("Expected PluginError due to cycle"),
    }
}

#[test]
fn test_invalid_task_config() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    // Invalid task (no command or dependencies)
    let script_content = r#"
    tasks:
      invalid_task:
        description: "This task has no command or dependencies."
    "#;

    fs::write(&script_path, script_content).unwrap();

    let mut loader = ScriptLoader::new();
    let mut config = BodoConfig::default();
    config.scripts_dirs = Some(vec![temp_dir.path().to_string_lossy().to_string()]);

    let result = loader.build_graph(config);

    assert!(
        result.is_err(),
        "Invalid task configuration was not detected"
    );
    match result {
        Err(BodoError::ValidationError(msg)) => {
            assert!(
                msg.contains("A task must have a command or some dependencies"),
                "Incorrect validation error message"
            );
        }
        _ => panic!("Expected ValidationError"),
    }
}

#[test]
fn test_parse_cross_file_ref() {
    let loader = ScriptLoader::new();
    let referencing_file = PathBuf::from("dir/script.yaml");
    let dep = "../other.yaml/some-task";
    let result = loader.parse_cross_file_ref(dep, &referencing_file);
    assert!(result.is_some());
    let (script_path, task_name) = result.unwrap();
    assert_eq!(script_path, PathBuf::from("dir/../other.yaml"));
    assert_eq!(task_name, "some-task".to_string());
}

#[test]
fn test_parse_cross_file_ref_no_slash() {
    let loader = ScriptLoader::new();
    let referencing_file = PathBuf::from("dir/script.yaml");
    let dep = "task-name";
    let result = loader.parse_cross_file_ref(dep, &referencing_file);
    assert!(result.is_none());
}
