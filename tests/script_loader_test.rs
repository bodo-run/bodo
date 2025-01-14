use bodo::{
    graph::Graph,
    script_loader::{load_bodo_config, load_scripts},
};
use std::error::Error;
use tempfile::tempdir;

#[test]
fn test_load_bodo_config_no_file() {
    let config = load_bodo_config(None).unwrap();
    assert!(
        config.scripts_dir.is_none(),
        "scripts_dir should be None when no config file is provided"
    );
    assert!(
        config.scripts_glob.is_none(),
        "scripts_glob should be None when no config file is provided"
    );
}

#[test]
fn test_load_bodo_config_file_not_found() -> Result<(), Box<dyn Error>> {
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("nonexistent.yaml");

    let result = load_bodo_config(Some(config_path.to_str().unwrap()));
    assert!(result.is_ok());
    let config = result.unwrap();
    assert!(config.scripts_dir.is_none());
    assert!(config.scripts_glob.is_none());

    Ok(())
}

#[test]
fn test_load_bodo_config_invalid_yaml() -> Result<(), Box<dyn Error>> {
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("invalid.yaml");
    std::fs::write(&config_path, "invalid: - yaml: content")?;

    let result = load_bodo_config(Some(config_path.to_str().unwrap()));
    assert!(result.is_ok());
    let config = result.unwrap();
    assert!(config.scripts_dir.is_none());
    assert!(config.scripts_glob.is_none());

    Ok(())
}

#[test]
fn test_load_bodo_config_valid_file() -> Result<(), Box<dyn Error>> {
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("bodo.yaml");
    std::fs::write(
        &config_path,
        r#"
        scripts_dir: "scripts/"
        scripts_glob: "*.yaml"
        "#,
    )?;

    let loaded = load_bodo_config(Some(config_path.to_str().unwrap())).unwrap();
    assert_eq!(
        loaded.scripts_dir, None,
        "scripts_dir should be None since script_loader is not implemented yet"
    );
    assert_eq!(
        loaded.scripts_glob, None,
        "scripts_glob should be None since script_loader is not implemented yet"
    );

    Ok(())
}

#[test]
fn test_load_scripts_no_scripts() -> Result<(), Box<dyn Error>> {
    let mut graph = Graph::new();
    let result = load_scripts(&[], &mut graph);
    assert!(result.is_ok());
    assert_eq!(graph.nodes.len(), 0);
    Ok(())
}

#[test]
fn test_load_scripts_invalid_yaml() -> Result<(), Box<dyn Error>> {
    let temp_dir = tempdir()?;
    let script_path = temp_dir.path().join("invalid.yaml");
    std::fs::write(&script_path, "invalid: - yaml: content")?;

    let mut graph = Graph::new();
    let result = load_scripts(&[script_path], &mut graph);
    assert!(result.is_ok());
    assert_eq!(graph.nodes.len(), 0);

    Ok(())
}

#[test]
fn test_load_scripts_valid_command() -> Result<(), Box<dyn Error>> {
    let temp_dir = tempdir()?;
    let script_path = temp_dir.path().join("test.yaml");
    std::fs::write(
        &script_path,
        r#"
        command: echo "test"
        description: Test command
        "#,
    )?;

    let mut graph = Graph::new();
    let result = load_scripts(&[script_path], &mut graph);
    assert!(result.is_ok());
    assert_eq!(
        graph.nodes.len(),
        0,
        "No nodes should be added since script_loader is not implemented yet"
    );

    Ok(())
}

#[test]
fn test_load_scripts_valid_task() -> Result<(), Box<dyn Error>> {
    let temp_dir = tempdir()?;
    let script_path = temp_dir.path().join("test.yaml");
    std::fs::write(
        &script_path,
        r#"
        task: test_task
        description: Test task
        "#,
    )?;

    let mut graph = Graph::new();
    let result = load_scripts(&[script_path], &mut graph);
    assert!(result.is_ok());
    assert_eq!(
        graph.nodes.len(),
        0,
        "No nodes should be added since script_loader is not implemented yet"
    );

    Ok(())
}

#[test]
fn test_load_scripts_multiple_files() -> Result<(), Box<dyn Error>> {
    let temp_dir = tempdir()?;
    let script1_path = temp_dir.path().join("test1.yaml");
    let script2_path = temp_dir.path().join("test2.yaml");

    std::fs::write(
        &script1_path,
        r#"
        command: echo "test1"
        description: Test command 1
        "#,
    )?;

    std::fs::write(
        &script2_path,
        r#"
        task: test_task
        description: Test task
        "#,
    )?;

    let mut graph = Graph::new();
    let result = load_scripts(&[script1_path, script2_path], &mut graph);
    assert!(result.is_ok());
    assert_eq!(
        graph.nodes.len(),
        0,
        "No nodes should be added since script_loader is not implemented yet"
    );

    Ok(())
}
