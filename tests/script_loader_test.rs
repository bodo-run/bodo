use bodo::config::{BodoConfig, TaskConfig};
use bodo::graph::{Graph, NodeKind, TaskData};
use bodo::script_loader::ScriptLoader;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_load_script() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    // Use a simple valid YAML without extra indentation.
    let script_content = r#"tasks:
  test_task:
    command: echo "Test Task"
"#;
    fs::write(&script_path, script_content).unwrap();

    let mut loader = ScriptLoader::new();
    let mut config = BodoConfig::default();
    // Use the directory containing the script.
    config.scripts_dirs = Some(vec![temp_dir.path().to_str().unwrap().to_string()]);

    let graph = loader.build_graph(config).unwrap();

    let script_id = script_path.to_str().unwrap();
    let full_task_name = format!("{} {}", script_id, "test_task");

    assert!(
        graph.task_registry.contains_key(&full_task_name),
        "Expected task registry to contain key '{}'",
        full_task_name
    );
}

#[test]
fn test_load_script_with_arguments_and_concurrently() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    let script_content = r#"tasks:
  task_with_args:
    command: echo "Hello $name"
    args:
      - name: name
        required: true
    env:
      GREETING: "Hello"
  concurrent_task:
    concurrently_options:
      fail_fast: true
    concurrently:
      - task: task_with_args
      - command: echo "Running concurrent command"
"#;
    fs::write(&script_path, script_content).unwrap();
    let mut loader = ScriptLoader::new();
    let mut config = BodoConfig::default();
    config.scripts_dirs = Some(vec![temp_dir.path().to_str().unwrap().to_string()]);
    let graph = loader.build_graph(config).unwrap();
    let script_id = script_path.to_str().unwrap().to_string();

    let full_task_name = format!("{} {}", script_id, "task_with_args");
    assert!(
        graph.task_registry.contains_key(&full_task_name),
        "Task 'task_with_args' not found in task registry"
    );
}

#[test]
fn test_build_graph_with_root_script_and_config_tasks() {
    let temp_dir = tempdir().unwrap();
    let root_script_path = temp_dir.path().join("root_script.yaml");

    let root_script_content = r#"tasks:
  root_task:
    command: echo "Root task"
"#;
    fs::write(&root_script_path, root_script_content).unwrap();

    let mut loader = ScriptLoader::new();
    let mut config = BodoConfig::default();
    config.root_script = Some(root_script_path.to_str().unwrap().to_string());
    config.default_task = Some(TaskConfig {
        command: Some("echo 'Default Task in Config'".to_string()),
        ..Default::default()
    });
    config.tasks.insert(
        "config_task".to_string(),
        TaskConfig {
            command: Some("echo 'Config Task'".to_string()),
            ..Default::default()
        },
    );
    config.scripts_dirs = Some(vec![temp_dir.path().to_str().unwrap().to_string()]);
    let graph = loader.build_graph(config).unwrap();

    let full_root_task_name = format!("{} {}", root_script_path.to_str().unwrap(), "root_task");
    assert!(
        graph.task_registry.contains_key(&full_root_task_name),
        "Expected 'root_task' from root_script to be loaded"
    );
    // Ensure tasks from config are not loaded when root_script exists.
    assert!(
        !graph.task_registry.contains_key("default"),
        "Did not expect 'default' task from config"
    );
    assert!(
        !graph.task_registry.contains_key("config_task"),
        "Did not expect 'config_task' from config"
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
    config.scripts_dirs = Some(vec![scripts_dir.to_str().unwrap().to_string()]);
    let graph = loader.build_graph(config).unwrap();

    let script_id1 = script1_path.to_str().unwrap().to_string();
    let full_task_name1 = format!("{} {}", script_id1, "task1");
    let script_id2 = script2_path.to_str().unwrap().to_string();
    let full_task_name2 = format!("{} {}", script_id2, "task2");

    assert!(
        graph.task_registry.contains_key(&full_task_name1),
        "Task1 not found in task registry"
    );
    assert!(
        graph.task_registry.contains_key(&full_task_name2),
        "Task2 not found in task registry"
    );
}

#[test]
fn test_task_dependencies() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");
    let script_content = r#"tasks:
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
    config.scripts_dirs = Some(vec![temp_dir.path().to_str().unwrap().to_string()]);
    let graph = loader.build_graph(config).unwrap();
    let script_id = script_path.to_str().unwrap().to_string();

    let full_task1_name = format!("{} {}", script_id, "task1");
    let full_task2_name = format!("{} {}", script_id, "task2");

    assert!(
        graph.task_registry.contains_key(&full_task1_name),
        "Task1 not found in registry"
    );
    assert!(
        graph.task_registry.contains_key(&full_task2_name),
        "Task2 not found in registry"
    );

    let task1_node = graph
        .nodes
        .iter()
        .find(|node| {
            if let NodeKind::Task(task) = &node.kind {
                task.name == "task1"
            } else {
                false
            }
        })
        .expect("Task1 not found");
    if let NodeKind::Task(task) = &task1_node.kind {
        assert!(
            !task.pre_deps.is_empty(),
            "Expected non-empty pre_deps for task1"
        );
    }
}
