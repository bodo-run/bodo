use bodo::manager::GraphManager;
use std::error::Error;
use tempfile::tempdir;
use tokio::test;

#[test]
async fn test_new_manager_is_empty() {
    let mgr = GraphManager::new();
    assert_eq!(mgr.graph.nodes.len(), 0);
    assert_eq!(mgr.graph.edges.len(), 0);
    assert!(mgr.config.scripts_dir.is_none());
    assert!(mgr.config.scripts_glob.is_none());
}

#[test]
async fn test_load_bodo_config() -> Result<(), Box<dyn Error>> {
    let mut mgr = GraphManager::new();
    let result = mgr.load_bodo_config().await;
    assert!(result.is_ok());
    assert!(mgr.config.scripts_dir.is_none());
    assert!(mgr.config.scripts_glob.is_none());
    Ok(())
}

#[test]
async fn test_build_graph_with_valid_yaml() -> Result<(), Box<dyn Error>> {
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

    let mut mgr = GraphManager::new();
    mgr.config.scripts_dir = Some(scripts_dir.to_string_lossy().into_owned());
    mgr.config.scripts_glob = Some("*.yml".to_string());

    let result = mgr.build_graph(&[script_path]).await;
    assert!(result.is_ok(), "build_graph should succeed");
    assert_eq!(
        mgr.graph.nodes.len(),
        0,
        "No nodes should be added since script_loader is not implemented yet"
    );

    Ok(())
}

#[test]
async fn test_build_graph_with_invalid_yaml() -> Result<(), Box<dyn Error>> {
    let temp_dir = tempdir()?;
    let scripts_dir = temp_dir.path().join("scripts");
    std::fs::create_dir_all(&scripts_dir)?;

    let script_path = scripts_dir.join("invalid.yml");
    std::fs::write(&script_path, "invalid: - yaml: content")?;

    let mut mgr = GraphManager::new();
    mgr.config.scripts_dir = Some(scripts_dir.to_string_lossy().into_owned());
    mgr.config.scripts_glob = Some("*.yml".to_string());

    let result = mgr.build_graph(&[script_path]).await;
    assert!(
        result.is_ok(),
        "build_graph should silently ignore invalid YAML"
    );
    assert_eq!(
        mgr.graph.nodes.len(),
        0,
        "No nodes should be added for invalid YAML"
    );

    Ok(())
}
