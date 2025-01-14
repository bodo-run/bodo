use bodo::{GraphManager, NodeKind};
use std::{
    error::Error,
    fs::{create_dir_all, write},
    path::Path,
};
use tempfile::tempdir;

use bodo::script_loader::BodoConfig;

/// Helper function: creates a minimal script file
fn write_minimal_script_file(
    dir: &Path,
    filename: &str,
    content: &str,
) -> Result<(), Box<dyn Error>> {
    let path = dir.join(filename);
    write(path, content)?;
    Ok(())
}

#[test]
fn test_end_to_end_minimal_project() -> Result<(), Box<dyn Error>> {
    let temp = tempdir()?;
    let root_path = temp.path();

    let scripts_dir = root_path.join("scripts");
    create_dir_all(&scripts_dir)?;
    let script_file_content = r#"
default_task:
  command: "echo 'Hello, World!'"
"#;
    write_minimal_script_file(&scripts_dir, "script.yaml", script_file_content)?;

    std::env::set_current_dir(root_path)?;

    let mut manager = GraphManager::new();
    manager.config.script_paths = Some(vec![scripts_dir.to_string_lossy().into_owned()]);
    manager.build_graph()?;

    assert_eq!(manager.graph.nodes.len(), 1);
    match &manager.graph.nodes[0].kind {
        NodeKind::Command(cmd) => {
            assert_eq!(cmd.raw_command, "echo 'Hello, World!'");
        }
        _ => panic!("Expected a Command node for the default_task"),
    }

    manager.debug_graph();
    Ok(())
}

#[test]
fn test_end_to_end_custom_paths_in_bodo_toml() -> Result<(), Box<dyn Error>> {
    let temp = tempdir()?;
    let root_path = temp.path();

    let custom_scripts_dir = root_path.join("custom-scripts");
    create_dir_all(&custom_scripts_dir)?;

    let script_a = r#"
default_task:
  command: "echo 'From A'"
tasks:
  foo:
    command: "echo 'foo'"
"#;
    let script_b = r#"
default_task:
  command: "echo 'From B'"
tasks:
  bar:
    command: "echo 'bar'"
"#;
    write_minimal_script_file(&custom_scripts_dir, "scriptA.yaml", script_a)?;
    write_minimal_script_file(&custom_scripts_dir, "scriptB.yaml", script_b)?;

    std::env::set_current_dir(root_path)?;

    let mut manager = GraphManager::new();
    manager.config.script_paths = Some(vec![custom_scripts_dir.to_string_lossy().into_owned()]);
    manager.build_graph()?;

    assert_eq!(manager.graph.nodes.len(), 4);

    let mut cmd_count = 0;
    let mut task_count = 0;
    for node in &manager.graph.nodes {
        match &node.kind {
            NodeKind::Command(_) => cmd_count += 1,
            NodeKind::Task(_) => task_count += 1,
        }
    }
    assert_eq!(cmd_count, 2, "Two default_task commands");
    assert_eq!(task_count, 2, "Two named tasks");

    manager.debug_graph();
    Ok(())
}

#[test]
fn test_end_to_end_mixed_files_in_scripts_dir() -> Result<(), Box<dyn Error>> {
    let temp = tempdir()?;
    let root_path = temp.path();

    let scripts_dir = root_path.join("scripts");
    create_dir_all(&scripts_dir)?;

    let script1 = r#"
default_task:
  command: "echo 'One'"
"#;
    write_minimal_script_file(&scripts_dir, "script1.yaml", script1)?;

    let script2 = r#"
default_task:
  command: "echo 'Two'"
tasks:
  test:
    command: "cargo test"
  lint:
    command: "cargo clippy"
"#;
    write_minimal_script_file(&scripts_dir, "script2.yaml", script2)?;

    std::env::set_current_dir(root_path)?;

    let mut manager = GraphManager::new();
    manager.config.script_paths = Some(vec![scripts_dir.to_string_lossy().into_owned()]);
    manager.build_graph()?;

    assert_eq!(manager.graph.nodes.len(), 4);

    let commands = manager
        .graph
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Command(_)))
        .count();
    let tasks = manager
        .graph
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Task(_)))
        .count();
    assert_eq!(commands, 2);
    assert_eq!(tasks, 2);

    manager.debug_graph();
    Ok(())
}

#[test]
fn test_end_to_end_invalid_yaml_stops_build() -> Result<(), Box<dyn Error>> {
    let temp = tempdir()?;
    let root_path = temp.path();

    let scripts_dir = root_path.join("scripts");
    create_dir_all(&scripts_dir)?;

    let good_content = r#"
default_task:
  command: "echo Good"
"#;
    write_minimal_script_file(&scripts_dir, "good.yaml", good_content)?;

    let bad_content = r#"
default_task:
  command: "echo Unclosed
"#;
    write_minimal_script_file(&scripts_dir, "bad.yaml", bad_content)?;

    std::env::set_current_dir(root_path)?;

    let mut manager = GraphManager::new();
    manager.config.script_paths = Some(vec![scripts_dir.to_string_lossy().into_owned()]);

    let result = manager.build_graph();
    assert!(result.is_err(), "Should fail due to invalid YAML parse");
    Ok(())
}

#[test]
fn test_end_to_end_no_scripts_found_is_okay() -> Result<(), Box<dyn Error>> {
    let temp = tempdir()?;
    let root_path = temp.path();

    // Create an empty scripts directory with a different name
    let scripts_dir = root_path.join("test-scripts");
    create_dir_all(&scripts_dir)?;

    std::env::set_current_dir(root_path)?;

    // Create a custom config to use the test-scripts directory
    let config = BodoConfig {
        script_paths: Some(vec!["test-scripts".to_string()]),
    };

    let mut manager = GraphManager::new();
    manager.load_bodo_config(None)?;
    manager.config = config;

    let result = manager.build_graph();
    assert!(result.is_ok());
    assert_eq!(manager.graph.nodes.len(), 0);

    manager.debug_graph();
    Ok(())
}

#[test]
fn test_end_to_end_custom_glob() -> Result<(), Box<dyn Error>> {
    let temp = tempdir()?;
    let root_path = temp.path();

    let scripts_dir = root_path.join("my-scripts");
    create_dir_all(&scripts_dir)?;

    write_minimal_script_file(
        &scripts_dir,
        "file1.yaml",
        r#"
default_task:
  command: 'echo 1'
"#,
    )?;
    write_minimal_script_file(
        &scripts_dir,
        "file2.yaml",
        r#"
default_task:
  command: 'echo 2'
"#,
    )?;

    std::env::set_current_dir(root_path)?;

    let mut manager = GraphManager::new();
    manager.config.script_paths = Some(vec![scripts_dir.to_string_lossy().into_owned()]);
    manager.build_graph()?;

    assert_eq!(manager.graph.nodes.len(), 2);
    manager.debug_graph();
    Ok(())
}
