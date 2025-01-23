use std::collections::HashMap;

use bodo::{
    config::BodoConfig, manager::GraphManager, plugins::print_list_plugin::PrintListPlugin, Result,
};

#[tokio::test]
async fn test_load_tasks() -> Result<()> {
    let mut manager = GraphManager::new();
    let config = BodoConfig {
        root_script: Some("scripts/basic.yaml".into()),
        scripts_dirs: Some(vec!["scripts/".into()]),
        tasks: HashMap::new(),
    };

    manager.build_graph(config).await?;
    let tasks = manager.get_tasks();
    assert!(!tasks.is_empty(), "Should have loaded tasks");

    let build_task = manager
        .get_task_by_name("build")
        .expect("build task should exist");
    assert_eq!(build_task.description.as_ref().unwrap(), "Build project");
    assert_eq!(build_task.command.as_ref().unwrap(), "cargo build");

    let test_task = manager
        .get_task_by_name("test")
        .expect("test task should exist");
    assert_eq!(test_task.description.as_ref().unwrap(), "Run tests");
    assert_eq!(test_task.command.as_ref().unwrap(), "cargo test");

    Ok(())
}

#[tokio::test]
async fn test_list_plugin() -> Result<()> {
    let mut manager = GraphManager::new();
    let config = BodoConfig {
        root_script: Some("scripts/basic.yaml".into()),
        scripts_dirs: Some(vec!["scripts/".into()]),
        tasks: HashMap::new(),
    };

    manager.build_graph(config).await?;
    manager.register_plugin(Box::new(PrintListPlugin));
    manager.run_plugins(None).await?;

    Ok(())
}
