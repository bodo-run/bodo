use bodo::config::BodoConfig;
use bodo::script_loader::ScriptLoader;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_load_script() {
    // Use a config YAML with root_script: so that tasks are loaded using the default branch.
    let config_yaml = r#"
default_task:
  command: echo "Test Task"
  description: Default task

tasks:
  test_task:
    command: echo "Test Task"
"#;
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");
    fs::write(&script_path, config_yaml).expect("Failed to write script file");

    let mut loader = ScriptLoader::new();
    let config = BodoConfig {
        root_script: Some(script_path.to_str().unwrap().to_string()),
        ..Default::default()
    };
    let graph = loader.build_graph(config).expect("Failed to build graph");
    // Since root_script was provided, tasks are registered with the key: "<root_script> <task_name>"
    let expected_key = format!("{} {}", script_path.to_str().unwrap(), "test_task");
    assert!(
        graph.task_registry.contains_key(&expected_key),
        "Expected task registry to contain key \"{}\"",
        expected_key
    );
}

#[test]
fn test_load_script_with_arguments_and_concurrently() {
    let config_yaml = r#"
default_task:
  command: echo "Default Task"
  description: Default task

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
    let config: BodoConfig =
        serde_yaml::from_str(config_yaml).expect("Failed to deserialize config");
    let mut loader = ScriptLoader::new();
    // Here we simulate tasks defined directly (root_script not provided)
    let graph = loader.build_graph(config).expect("Failed to build graph");
    assert!(graph.task_registry.contains_key("task_with_args"));
}

#[test]
fn test_load_scripts_dir() {
    let temp_dir = tempdir().unwrap();
    let scripts_dir = temp_dir.path().join("scripts");
    fs::create_dir_all(&scripts_dir).unwrap();

    let script1_path = scripts_dir.join("script1.yaml");
    let script2_path = scripts_dir.join("script2.yaml");

    let script1_content = r#"
default_task:
  command: echo "Task1"
tasks:
  task1:
    command: echo "Task1"
"#;

    let script2_content = r#"
default_task:
  command: echo "Task2"
tasks:
  task2:
    command: echo "Task2"
"#;

    fs::write(&script1_path, script1_content).unwrap();
    fs::write(&script2_path, script2_content).unwrap();

    let mut loader = ScriptLoader::new();
    let config = BodoConfig {
        root_script: Some(script1_path.to_str().unwrap().to_string()),
        ..Default::default()
    };
    let graph = loader
        .build_graph(config)
        .expect("Failed to build graph from root_script");
    let expected_key = format!("{} {}", script1_path.to_str().unwrap(), "task1");
    assert!(
        graph.task_registry.contains_key(&expected_key),
        "Task1 not found in task registry"
    );
}
