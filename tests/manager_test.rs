use std::fs::{create_dir_all, write};
use tempfile::tempdir;

use bodo::{errors::PluginError, graph::NodeKind, manager::GraphManager};

#[test]
fn test_new_manager() {
    let mgr = GraphManager::new();
    assert_eq!(
        mgr.graph.nodes.len(),
        0,
        "New manager's graph should be empty"
    );
    assert_eq!(mgr.graph.edges.len(), 0);
    assert!(
        mgr.config.script_paths.is_none(),
        "Default config has no script_paths"
    );
}

#[test]
fn test_load_bodo_config_no_file() {
    let mut mgr = GraphManager::new();
    // Pass None => tries bodo.toml in current dir, which is presumably missing.
    let result = mgr.load_bodo_config(None);
    assert!(result.is_ok());
    // Confirm default config
    assert!(mgr.config.script_paths.is_none());
}

#[test]
fn test_load_bodo_config_valid_file() {
    // Create a temp dir with a minimal bodo.toml
    let temp = tempdir().unwrap();
    let config_path = temp.path().join("bodo.toml");

    write(&config_path, r#"script_paths = ["my-scripts/"]"#).unwrap();

    let mut mgr = GraphManager::new();
    // Provide the path
    let result = mgr.load_bodo_config(Some(config_path.to_string_lossy().as_ref()));
    assert!(result.is_ok());

    // Confirm it was parsed
    assert_eq!(
        mgr.config.script_paths,
        Some(vec!["my-scripts/".to_string()])
    );
}

#[test]
fn test_build_graph_with_scripts() {
    let temp = tempdir().unwrap();
    let scripts_dir = temp.path().join("scripts");
    create_dir_all(&scripts_dir).unwrap();

    // Write two script files
    let file_a = scripts_dir.join("scriptA.yaml");
    let file_b = scripts_dir.join("scriptB.yaml");

    let yaml_a = r#"
default_task:
  command: "echo A"
tasks:
  alpha:
    command: "echo alpha"
"#;
    let yaml_b = r#"
default_task:
  command: "echo B"
tasks:
  beta:
    command: "echo beta"
"#;

    write(&file_a, yaml_a).unwrap();
    write(&file_b, yaml_b).unwrap();

    // Set config so it loads from scripts/
    let mut mgr = GraphManager::new();
    mgr.config.script_paths = Some(vec![scripts_dir.to_string_lossy().into_owned()]);

    let result = mgr.build_graph();
    assert!(result.is_ok(), "build_graph should succeed");

    // 4 nodes total: each file has 1 default task + 1 named task.
    assert_eq!(mgr.graph.nodes.len(), 4);

    // Optionally confirm which are commands vs tasks
    let cmd_count = mgr
        .graph
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Command(_)))
        .count();
    let task_count = mgr
        .graph
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Task(_)))
        .count();
    assert_eq!(cmd_count, 2);
    assert_eq!(task_count, 2);
}

#[test]
fn test_build_graph_invalid_yaml() {
    let temp = tempdir().unwrap();
    let scripts_dir = temp.path().join("scripts");
    create_dir_all(&scripts_dir).unwrap();

    let bad_file = scripts_dir.join("bad.yaml");
    write(
        &bad_file,
        "default_task:\n  command: [this is not valid yaml}",
    )
    .unwrap();

    let mut mgr = GraphManager::new();
    mgr.config.script_paths = Some(vec![scripts_dir.to_string_lossy().into_owned()]);

    let result = mgr.build_graph();
    match result {
        Err(PluginError::GenericError(msg)) => {
            assert!(
                msg.contains("YAML parse error"),
                "Should mention parse error for invalid YAML"
            );
        }
        _ => panic!("Expected parse error from invalid YAML"),
    }
}

#[test]
fn test_build_graph_no_scripts_dir() {
    let temp = tempdir().unwrap();
    // We won't create a scripts dir at all
    let mut mgr = GraphManager::new();
    mgr.config.script_paths = Some(vec![temp
        .path()
        .join("scripts")
        .to_string_lossy()
        .into_owned()]);

    // Our default loader code just skips if path doesn't exist => no error, zero nodes
    let result = mgr.build_graph();
    assert!(result.is_ok());
    assert_eq!(mgr.graph.nodes.len(), 0);
}

#[test]
fn test_debug_graph_no_panic() {
    let mgr = GraphManager::new();
    // Graph is empty => just ensure it doesn't panic
    mgr.debug_graph();
}
