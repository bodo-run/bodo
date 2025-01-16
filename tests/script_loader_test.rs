use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

use bodo::{
    config::ConcurrentItem,
    graph::{Graph, NodeKind},
    script_loader::{load_bodo_config, load_scripts},
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

    assert_eq!(graph.nodes.len(), 4);

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
  dependencies:
    - task: "test"
tasks:
  test:
    command: "cargo test"
    dependencies:
      - command: "cargo build"
"#;
    let (path, _dir) = create_temp_script_file(content);
    let mut graph = Graph::new();
    load_scripts(&[path], &mut graph).unwrap();

    assert_eq!(graph.nodes.len(), 4);
    assert_eq!(graph.edges.len(), 2);

    let command_id = 0;
    let default_id = 1;
    let test_id = 2;

    assert!(graph
        .edges
        .iter()
        .any(|e| e.from == default_id && e.to == test_id));
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

    assert_eq!(graph.nodes.len(), 4);

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

    assert_eq!(graph.nodes.len(), 5);
    assert_eq!(graph.edges.len(), 1);
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
    assert_eq!(graph.nodes.len(), 4);
}

#[test]
fn test_load_bodo_config_empty() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("bodo.yaml");
    let content = r#"
scripts_dirs: null
root_script: null
"#;
    File::create(&config_path)
        .unwrap()
        .write_all(content.as_bytes())
        .unwrap();

    let config = load_bodo_config(Some(config_path.to_str().unwrap())).unwrap();
    assert_eq!(config.scripts_dirs, None);
    assert_eq!(config.root_script, None);
}
