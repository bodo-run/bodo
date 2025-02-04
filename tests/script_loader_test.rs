use bodo::config::BodoConfig;
use bodo::script_loader::ScriptLoader;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_load_script() {
    // Use a config YAML without a root_script (so tasks use their key as task_name)
    let config_yaml = r#"
tasks:
  test_task:
    command: echo "Test Task"
"#;
    let config: BodoConfig = serde_yaml::from_str(config_yaml).unwrap();
    let mut loader = ScriptLoader::new();
    let graph = loader.build_graph(config).unwrap();
    // Since root_script is None, task key is "test_task"
    assert!(
        graph.task_registry.contains_key("test_task"),
        "Expected task registry to contain key \"test_task\""
    );
}

#[test]
fn test_load_script_with_arguments_and_concurrently() {
    let config_yaml = r#"
tasks:
  task_with_args:
    command: echo "Hello ${name}"
    args:
      - name: name
        required: true
        default: "Alice"
    concurrently:
      - task: task_with_args
      - command: echo "Concurrent command"
"#;
    let config: BodoConfig = serde_yaml::from_str(config_yaml).unwrap();
    let mut loader = ScriptLoader::new();
    let graph = loader.build_graph(config).unwrap();
    // Since root_script is None, keys are plain task names.
    assert!(
        graph.task_registry.contains_key("task_with_args"),
        "Task 'task_with_args' not found in task registry"
    );
}

#[test]
fn test_load_scripts_dir() {
    let temp_dir = tempdir().unwrap();
    let scripts_dir = temp_dir.path().join("scripts");
    fs::create_dir_all(&scripts_dir).unwrap();

    let script1_path = scripts_dir.join("script1.yaml");
    let script2_path = scripts_dir.join("script2.yaml");

    let script1_content = r#"tasks:
  task1:
    command: echo "Task 1"
"#;

    let script2_content = r#"tasks:
  task2:
    command: echo "Task 2"
"#;

    fs::write(&script1_path, script1_content).unwrap();
    fs::write(&script2_path, script2_content).unwrap();

    let mut loader = ScriptLoader::new();
    let mut config = BodoConfig::default();
    // Set scripts_dirs so that build_graph will load tasks from these files.
    config.scripts_dirs = Some(vec![scripts_dir.to_str().unwrap().to_string()]);
    let graph = loader.build_graph(config).unwrap();

    // For scripts loaded via scripts_dirs, the loader should register tasks using the file path as key.
    let key1 = format!("{} {}", script1_path.to_str().unwrap(), "task1");
    let key2 = format!("{} {}", script2_path.to_str().unwrap(), "task2");
    assert!(
        graph.task_registry.contains_key(&key1),
        "Task1 not found in task registry"
    );
    assert!(
        graph.task_registry.contains_key(&key2),
        "Task2 not found in task registry"
    );
}

#[test]
fn test_task_dependencies() {
    let config_yaml = r#"
tasks:
  task1:
    command: echo "Task 1"
    pre_deps:
      - task: task2
  task2:
    command: echo "Task 2"
"#;
    let config: BodoConfig = serde_yaml::from_str(config_yaml).unwrap();
    let mut loader = ScriptLoader::new();
    let graph = loader.build_graph(config).unwrap();
    // Since root_script is None, keys are plain task names.
    assert!(
        graph.task_registry.contains_key("task1"),
        "Task1 not found in registry"
    );
    assert!(
        graph.task_registry.contains_key("task2"),
        "Task2 not found in registry"
    );
    // Additionally, fetch the task1 node and verify pre_deps length.
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
        .expect("Task1 not found");
    if let bodo::graph::NodeKind::Task(task) = &task1_node.kind {
        assert!(
            !task.pre_deps.is_empty(),
            "Expected non-empty pre_deps for task1"
        );
    }
}
