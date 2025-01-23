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
        .expect("Build task should exist");
    assert_eq!(
        build_task.description.as_deref(),
        Some("Build release binary")
    );
    assert_eq!(build_task.command.as_deref(), Some("cargo build --release"));

    let test_task = manager
        .get_task_by_name("test")
        .expect("Test task should exist");
    assert_eq!(test_task.description.as_deref(), Some("Run all tests"));
    assert_eq!(test_task.command.as_deref(), Some("cargo test --verbose"));

    let check_task = manager
        .get_task_by_name("check")
        .expect("Check task should exist");
    assert_eq!(check_task.description.as_deref(), Some("Run clippy checks"));
    assert_eq!(check_task.command.as_deref(), Some("cargo clippy"));

    let default_task = manager
        .get_default_task()
        .expect("Default task should exist");
    assert_eq!(
        default_task.description.as_deref(),
        Some("Default task example")
    );
    assert_eq!(
        default_task.command.as_deref(),
        Some("echo \"Running default task\"")
    );

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
