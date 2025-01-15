use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tempfile::tempdir;

use bodo::{
    config::ConcurrentItem,
    graph::{Graph, NodeKind},
    script_loader::{load_bodo_config, load_scripts, BodoConfig},
};

fn create_temp_script_file(content: &str) -> (PathBuf, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test_script.yaml");
    let mut file = File::create(&file_path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    (file_path, dir)
}

#[test]
fn test_load_simple_script() {
    let content = r#"
default_task:
  command: "echo hello"
  description: "Default task"
tasks:
  test:
    command: "cargo test"
    description: "Run tests"
"#;
    let (path, _dir) = create_temp_script_file(content);
    let mut graph = Graph::new();
    load_scripts(&[path], &mut graph).unwrap();

    assert_eq!(graph.nodes.len(), 2);

    let default_node = &graph.nodes[0];
    if let NodeKind::Task(task_data) = &default_node.kind {
        assert_eq!(task_data.command, Some("echo hello".to_string()));
        assert_eq!(task_data.description, Some("Default task".to_string()));
    } else {
        panic!("Expected Task node");
    }

    let test_node = &graph.nodes[1];
    if let NodeKind::Task(task_data) = &test_node.kind {
        assert_eq!(task_data.command, Some("cargo test".to_string()));
        assert_eq!(task_data.description, Some("Run tests".to_string()));
    } else {
        panic!("Expected Task node");
    }
}

#[test]
fn test_load_script_with_dependencies() {
    let content = r#"
default_task:
  command: "echo hello"
  pre_deps:
    - task: "test"
tasks:
  test:
    command: "cargo test"
    pre_deps:
      - command: "cargo build"
"#;
    let (path, _dir) = create_temp_script_file(content);
    let mut graph = Graph::new();
    load_scripts(&[path], &mut graph).unwrap();

    assert_eq!(graph.nodes.len(), 3);
    assert_eq!(graph.edges.len(), 2);

    let default_id = 0;
    let test_id = 1;
    let command_id = 2;

    assert!(graph
        .edges
        .iter()
        .any(|e| e.from == test_id && e.to == default_id));
    assert!(graph
        .edges
        .iter()
        .any(|e| e.from == command_id && e.to == test_id));
}

#[test]
fn test_load_script_with_concurrency() {
    let content = r#"
default_task:
  command: "echo hello"
  concurrently:
    - command: "watch files"
    - task: "test"
tasks:
  test:
    command: "cargo test"
"#;
    let (path, _dir) = create_temp_script_file(content);
    let mut graph = Graph::new();
    load_scripts(&[path], &mut graph).unwrap();

    assert_eq!(graph.nodes.len(), 2);

    let default_node = &graph.nodes[0];
    let concurrency = default_node.metadata.get("concurrently").unwrap();
    let items: Vec<ConcurrentItem> = serde_json::from_str(concurrency).unwrap();
    assert_eq!(items.len(), 2);
}

#[test]
fn test_load_multiple_scripts() {
    let content1 = r#"
default_task:
  command: "echo hello"
tasks:
  test:
    command: "cargo test"
"#;
    let content2 = r#"
default_task:
  command: "echo world"
  pre_deps:
    - task: "test_script#test"
"#;
    let dir = tempdir().unwrap();
    let path1 = dir.path().join("test_script.yaml");
    let path2 = dir.path().join("other_script.yaml");

    File::create(&path1)
        .unwrap()
        .write_all(content1.as_bytes())
        .unwrap();
    File::create(&path2)
        .unwrap()
        .write_all(content2.as_bytes())
        .unwrap();

    let mut graph = Graph::new();
    load_scripts(&[path1, path2], &mut graph).unwrap();

    assert_eq!(graph.nodes.len(), 3);
    assert_eq!(graph.edges.len(), 1);
}

#[test]
fn test_load_invalid_dependency() {
    let content = r#"
default_task:
  command: "echo hello"
  pre_deps:
    - task: "nonexistent"
"#;
    let (path, _dir) = create_temp_script_file(content);
    let mut graph = Graph::new();
    let result = load_scripts(&[path], &mut graph);
    assert!(result.is_err());
}

#[test]
fn test_load_script_with_env_vars() {
    let content = r#"
default_task:
  command: "echo hello"
  env:
    FOO: "bar"
    DEBUG: "true"
"#;
    let (path, _dir) = create_temp_script_file(content);
    let mut graph = Graph::new();
    load_scripts(&[path], &mut graph).unwrap();

    let node = &graph.nodes[0];
    let env_json = node.metadata.get("env").unwrap();
    let env_map: HashMap<String, String> = serde_json::from_str(env_json).unwrap();
    assert_eq!(env_map.get("FOO").unwrap(), "bar");
    assert_eq!(env_map.get("DEBUG").unwrap(), "true");
}

#[test]
fn test_load_script_with_output_config() {
    let content = r#"
default_task:
  command: "echo hello"
  output:
    prefix: "test"
    color: "Blue"
    disable_color: true
"#;
    let (path, _dir) = create_temp_script_file(content);
    let mut graph = Graph::new();
    load_scripts(&[path], &mut graph).unwrap();

    let node = &graph.nodes[0];
    let output_json = node.metadata.get("output").unwrap();
    assert!(output_json.contains("\"prefix\":\"test\""));
    assert!(output_json.contains("\"color\":\"Blue\""));
    assert!(output_json.contains("\"disable_color\":true"));
}

#[test]
fn test_load_script_with_circular_dependency() {
    let content = r#"
default_task:
  command: "echo hello"
  pre_deps:
    - task: "test"
tasks:
  test:
    command: "cargo test"
    pre_deps:
      - task: "default"
"#;
    let (path, _dir) = create_temp_script_file(content);
    let mut graph = Graph::new();
    let result = load_scripts(&[path], &mut graph);
    assert!(result.is_err());
}

#[test]
fn test_load_script_with_missing_file() {
    let mut graph = Graph::new();
    let nonexistent = PathBuf::from("nonexistent.yaml");
    let result = load_scripts(&[nonexistent], &mut graph);
    assert!(result.is_ok());
    assert_eq!(graph.nodes.len(), 0);
}

#[test]
fn test_load_script_with_invalid_yaml() {
    let content = "invalid: - yaml: content:";
    let (path, _dir) = create_temp_script_file(content);
    let mut graph = Graph::new();
    let result = load_scripts(&[path], &mut graph);
    assert!(result.is_ok());
    assert_eq!(graph.nodes.len(), 0);
}

#[test]
fn test_load_script_with_empty_tasks() {
    let content = r#"
default_task:
  command: "echo hello"
tasks: {}
"#;
    let (path, _dir) = create_temp_script_file(content);
    let mut graph = Graph::new();
    load_scripts(&[path], &mut graph).unwrap();
    assert_eq!(graph.nodes.len(), 1);
}

#[test]
fn test_load_script_with_working_dir() {
    let content = r#"
default_task:
  command: "echo hello"
  cwd: "/tmp/test"
"#;
    let (path, _dir) = create_temp_script_file(content);
    let mut graph = Graph::new();
    load_scripts(&[path], &mut graph).unwrap();

    let node = &graph.nodes[0];
    if let NodeKind::Task(task_data) = &node.kind {
        assert_eq!(task_data.working_dir, Some("/tmp/test".to_string()));
    }
}

#[test]
fn test_load_bodo_config() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("bodo.yaml");
    let content = r#"
scripts_dir: "scripts"
scripts_glob: "*.yaml"
"#;
    File::create(&config_path)
        .unwrap()
        .write_all(content.as_bytes())
        .unwrap();

    let config = load_bodo_config(Some(config_path.to_str().unwrap())).unwrap();
    assert_eq!(config.scripts_dir, Some("scripts".to_string()));
    assert_eq!(config.scripts_glob, Some("*.yaml".to_string()));
}

#[test]
fn test_load_bodo_config_invalid_path() {
    let config = load_bodo_config(Some("nonexistent.yaml")).unwrap();
    assert_eq!(config, BodoConfig::default());
}

#[test]
fn test_load_bodo_config_invalid_yaml() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("bodo.yaml");
    let content = "invalid: - yaml: content:";
    File::create(&config_path)
        .unwrap()
        .write_all(content.as_bytes())
        .unwrap();

    let config = load_bodo_config(Some(config_path.to_str().unwrap())).unwrap();
    assert_eq!(config, BodoConfig::default());
}

#[test]
fn test_load_bodo_config_empty() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("bodo.yaml");
    let content = "";
    File::create(&config_path)
        .unwrap()
        .write_all(content.as_bytes())
        .unwrap();

    let config = load_bodo_config(Some(config_path.to_str().unwrap())).unwrap();
    assert_eq!(config, BodoConfig::default());
}
