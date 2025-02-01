use std::collections::HashMap;
use std::fs;
use tempfile::tempdir;

use bodo::{
    config::BodoConfig, manager::GraphManager, plugins::print_list_plugin::PrintListPlugin, Result,
};

#[tokio::test]
async fn test_load_tasks() -> Result<()> {
    let temp_dir = tempdir()?;
    let scripts_dir = temp_dir.path().join("scripts");
    fs::create_dir_all(&scripts_dir)?;

    // Create test script file
    fs::write(
        scripts_dir.join("build.yaml"),
        r#"
tasks:
  build:
    command: echo "Building project"
    description: "Build task"
"#,
    )?;

    let mut manager = GraphManager::new();
    let config = BodoConfig {
        root_script: Some(
            scripts_dir
                .join("build.yaml")
                .to_string_lossy()
                .into_owned(),
        ),
        scripts_dirs: Some(vec![scripts_dir.to_string_lossy().into_owned()]),
        tasks: HashMap::new(),
    };

    manager.build_graph(config).await?;
    let tasks = manager.get_tasks();
    assert!(!tasks.is_empty(), "Should have loaded tasks");

    let build_task = manager
        .get_task_by_name("build")
        .expect("Build task should exist");
    assert_eq!(build_task.description.as_deref(), Some("Build task"));
    assert_eq!(
        build_task.command.as_deref(),
        Some("echo \"Building project\"")
    );

    Ok(())
}

#[tokio::test]
async fn test_list_plugin() -> Result<()> {
    let temp_dir = tempdir()?;
    let scripts_dir = temp_dir.path().join("scripts");
    fs::create_dir_all(&scripts_dir)?;

    // Create test script file
    fs::write(
        scripts_dir.join("basic.yaml"),
        r#"
tasks:
  test:
    command: echo "Running tests"
    description: "Test task"
"#,
    )?;

    let mut manager = GraphManager::new();
    let config = BodoConfig {
        root_script: Some(
            scripts_dir
                .join("basic.yaml")
                .to_string_lossy()
                .into_owned(),
        ),
        scripts_dirs: Some(vec![scripts_dir.to_string_lossy().into_owned()]),
        tasks: HashMap::new(),
    };

    manager.build_graph(config).await?;
    manager.register_plugin(Box::new(PrintListPlugin));
    manager.run_plugins(None).await?;

    Ok(())
}
