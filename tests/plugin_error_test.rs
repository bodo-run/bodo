use bodo::{
    errors::Result as BodoResult,
    graph::{Graph, NodeKind, TaskData},
    plugin::{PluginConfig, PluginManager},
    plugins::{
        failing_plugin::FakeFailingPlugin, path_plugin::PathPlugin, prefix_plugin::PrefixPlugin,
    },
};

#[tokio::test]
async fn test_one_plugin_fails_on_init_others_succeed() -> BodoResult<()> {
    // Set up a normal graph
    let mut graph = Graph::new();
    graph.add_node(NodeKind::Task(TaskData {
        name: "example".to_string(),
        description: None,
        command: Some("echo example".to_string()),
        working_dir: None,
    }));

    // Create plugin manager and register plugins
    let mut manager = PluginManager::new();
    manager.register(Box::new(PrefixPlugin::new()));
    manager.register(Box::new(FakeFailingPlugin::new(
        /*fail_on_init=*/ true, /*fail_on_graph_build=*/ false,
    )));
    manager.register(Box::new(PathPlugin::new()));

    let config = PluginConfig { options: None };

    // Initialize plugins - this should return an error
    let result = manager.init_plugins(&config).await;
    assert!(result.is_err(), "Expected plugin initialization to fail");

    Ok(())
}

#[tokio::test]
async fn test_one_plugin_fails_on_graph_build_others_succeed() -> BodoResult<()> {
    let mut graph = Graph::new();
    graph.add_node(NodeKind::Task(TaskData {
        name: "example".to_string(),
        description: None,
        command: Some("echo example".to_string()),
        working_dir: None,
    }));

    let mut manager = PluginManager::new();
    manager.register(Box::new(PrefixPlugin::new()));
    manager.register(Box::new(FakeFailingPlugin::new(
        /*fail_on_init=*/ false, /*fail_on_graph_build=*/ true,
    )));
    manager.register(Box::new(PathPlugin::new()));

    let config = PluginConfig { options: None };

    // Initialize plugins - should succeed
    manager.init_plugins(&config).await?;

    // Build graph - should fail
    let result = manager.on_graph_build(&mut graph).await;
    assert!(result.is_err(), "Expected graph build to fail");

    Ok(())
}
