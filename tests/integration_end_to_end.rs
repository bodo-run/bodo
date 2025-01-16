use bodo::manager::GraphManager;
use std::error::Error;
use tempfile::tempdir;
use tokio::test;

#[test]
async fn test_end_to_end_invalid_yaml() -> Result<(), Box<dyn Error>> {
    let temp_dir = tempdir()?;
    let scripts_dir = temp_dir.path().join("scripts");
    std::fs::create_dir_all(&scripts_dir)?;

    let script_path = scripts_dir.join("invalid.yaml");
    std::fs::write(&script_path, "invalid: - yaml: content")?;

    let mut manager = GraphManager::new();
    manager.config.scripts_dirs = Some(vec![scripts_dir.to_string_lossy().into_owned()]);
    manager.config.root_script = Some("*.yaml".to_string());

    let result = manager.build_graph(manager.config.clone()).await;
    assert!(result.is_ok(), "Should silently ignore invalid YAML");
    assert_eq!(
        manager.graph.nodes.len(),
        0,
        "No nodes should be added for invalid YAML"
    );

    Ok(())
}

#[test]
async fn test_end_to_end_no_scripts_found_is_okay() -> Result<(), Box<dyn Error>> {
    let temp_dir = tempdir()?;
    let scripts_dir = temp_dir.path().join("scripts");
    std::fs::create_dir_all(&scripts_dir)?;

    let mut manager = GraphManager::new();
    manager.config.scripts_dirs = Some(vec![scripts_dir.to_string_lossy().into_owned()]);
    manager.config.root_script = Some("*.yaml".to_string());

    let result = manager.build_graph(manager.config.clone()).await;
    assert!(result.is_ok());
    assert_eq!(manager.graph.nodes.len(), 0);

    Ok(())
}

#[test]
async fn test_end_to_end_custom_glob() -> Result<(), Box<dyn Error>> {
    let temp_dir = tempdir()?;
    let scripts_dir = temp_dir.path().join("scripts");
    std::fs::create_dir_all(&scripts_dir)?;

    let script_path = scripts_dir.join("test.yml");
    std::fs::write(
        &script_path,
        r#"
        command: echo "test"
        description: Test command
        "#,
    )?;

    let mut manager = GraphManager::new();
    manager.config.scripts_dirs = Some(vec![scripts_dir.to_string_lossy().into_owned()]);
    manager.config.root_script = Some("*.yml".to_string());

    let result = manager.build_graph(manager.config.clone()).await;
    assert!(result.is_ok());
    assert_eq!(
        manager.graph.nodes.len(),
        0,
        "No nodes should be added since script_loader is not implemented yet"
    );

    Ok(())
}
