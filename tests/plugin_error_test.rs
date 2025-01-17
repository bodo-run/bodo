use std::collections::HashMap;

use bodo::{
    errors::BodoError,
    graph::{Graph, NodeKind, TaskData},
    plugin::{PluginConfig, PluginManager},
    plugins::{
        failing_plugin::FakeFailingPlugin, path_plugin::PathPlugin, prefix_plugin::PrefixPlugin,
    },
    Result,
};

#[tokio::test]
async fn test_plugin_init_failure() -> Result<()> {
    let mut graph = Graph::new();
    graph.add_node(NodeKind::Task(TaskData {
        name: "test".to_string(),
        description: Some("Test task".to_string()),
        command: Some("echo test".to_string()),
        working_dir: None,
        is_default: false,
        script_name: Some("Test".to_string()),
        env: HashMap::new(),
    }));

    let mut manager = PluginManager::new();
    manager.register(Box::new(FakeFailingPlugin));

    let config = PluginConfig::default();
    let result = manager.run_lifecycle(&mut graph, &config).await;

    assert!(matches!(result, Err(BodoError::PluginError(_))));
    Ok(())
}

#[tokio::test]
async fn test_plugin_build_failure() -> Result<()> {
    let mut graph = Graph::new();
    graph.add_node(NodeKind::Task(TaskData {
        name: "test".to_string(),
        description: Some("Test task".to_string()),
        command: Some("echo test".to_string()),
        working_dir: None,
        is_default: false,
        script_name: Some("Test".to_string()),
        env: HashMap::new(),
    }));

    let mut manager = PluginManager::new();
    manager.register(Box::new(FakeFailingPlugin));

    let config = PluginConfig::default();
    let result = manager.run_lifecycle(&mut graph, &config).await;

    assert!(matches!(result, Err(BodoError::PluginError(_))));
    Ok(())
}

#[tokio::test]
async fn test_plugin_chain_failure() -> Result<()> {
    let mut graph = Graph::new();
    graph.add_node(NodeKind::Task(TaskData {
        name: "test".to_string(),
        description: Some("Test task".to_string()),
        command: Some("echo test".to_string()),
        working_dir: None,
        is_default: false,
        script_name: Some("Test".to_string()),
        env: HashMap::new(),
    }));

    let mut manager = PluginManager::new();
    manager.register(Box::new(PathPlugin::new()));
    manager.register(Box::new(PrefixPlugin::new()));
    manager.register(Box::new(FakeFailingPlugin));

    let config = PluginConfig::default();
    let result = manager.run_lifecycle(&mut graph, &config).await;

    assert!(matches!(result, Err(BodoError::PluginError(_))));
    Ok(())
}
