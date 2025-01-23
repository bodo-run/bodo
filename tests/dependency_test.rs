use bodo::{
    errors::BodoError,
    graph::{Graph, NodeKind, TaskData},
    plugin::{Plugin, PluginConfig, PluginManager},
    plugins::resolver_plugin::ResolverPlugin,
    Result,
};
use serde_json::json;
use std::collections::HashMap;

#[tokio::test]
async fn test_resolver_plugin() -> Result<()> {
    let mut graph = Graph::new();

    // Create task nodes
    let task1 = TaskData {
        name: "build".into(),
        description: Some("Build the project".into()),
        command: Some("echo Building".into()),
        working_dir: None,
        env: HashMap::new(),
        is_default: false,
        script_id: "build_script".to_string(),
        script_display_name: "Build Script".to_string(),
    };
    let task1_id = graph.add_node(NodeKind::Task(task1));
    graph
        .task_registry
        .insert("build_script build".to_string(), task1_id);

    let task2 = TaskData {
        name: "test".into(),
        description: Some("Test the project".into()),
        command: Some("echo Testing".into()),
        working_dir: None,
        env: HashMap::new(),
        is_default: false,
        script_id: "test_script".to_string(),
        script_display_name: "Test Script".to_string(),
    };
    let task2_id = graph.add_node(NodeKind::Task(task2));

    // Add dependency metadata
    let node = &mut graph.nodes[task2_id as usize];
    node.metadata.insert(
        "pre_deps".to_string(),
        json!(["build_script build"]).to_string(),
    );

    // Setup plugins
    let mut manager = PluginManager::new();
    manager.register(Box::new(ResolverPlugin));

    // Run plugins to process metadata
    manager
        .run_lifecycle(
            &mut graph,
            Some(PluginConfig {
                fail_fast: false,
                watch: false,
                list: false,
                options: None,
            }),
        )
        .await?;

    // Verify edge was created
    assert!(graph
        .edges
        .iter()
        .any(|e| e.from == task1_id && e.to == task2_id));

    Ok(())
}

#[tokio::test]
async fn test_resolver_plugin_missing_dependency() -> Result<()> {
    let mut graph = Graph::new();

    // Create task node
    let task = TaskData {
        name: "test".into(),
        description: Some("Test task".into()),
        command: Some("echo test".into()),
        working_dir: None,
        env: HashMap::new(),
        is_default: false,
        script_id: "test_script".to_string(),
        script_display_name: "Test Script".to_string(),
    };
    let task_id = graph.add_node(NodeKind::Task(task));

    // Add invalid dependency metadata
    let node = &mut graph.nodes[task_id as usize];
    node.metadata.insert(
        "pre_deps".to_string(),
        json!(["non_existent_task"]).to_string(),
    );

    // Setup plugins
    let mut manager = PluginManager::new();
    manager.register(Box::new(ResolverPlugin));

    // Run plugins to process metadata
    let result = manager
        .run_lifecycle(
            &mut graph,
            Some(PluginConfig {
                fail_fast: false,
                watch: false,
                list: false,
                options: None,
            }),
        )
        .await;

    assert!(result.is_err());
    assert!(matches!(result, Err(BodoError::PluginError(_))));

    Ok(())
}

#[tokio::test]
async fn test_resolver_plugin_invalid_metadata() -> Result<()> {
    let mut graph = Graph::new();

    // Create task node
    let task = TaskData {
        name: "test".into(),
        description: Some("Test task".into()),
        command: Some("echo test".into()),
        working_dir: None,
        env: HashMap::new(),
        is_default: false,
        script_id: "test_script".to_string(),
        script_display_name: "Test Script".to_string(),
    };
    let task_id = graph.add_node(NodeKind::Task(task));

    // Add invalid dependency metadata
    let node = &mut graph.nodes[task_id as usize];
    node.metadata
        .insert("pre_deps".to_string(), "invalid json".to_string());

    // Setup plugins
    let mut manager = PluginManager::new();
    manager.register(Box::new(ResolverPlugin));

    // Run plugins to process metadata
    let result = manager
        .run_lifecycle(
            &mut graph,
            Some(PluginConfig {
                fail_fast: false,
                watch: false,
                list: false,
                options: None,
            }),
        )
        .await;

    assert!(result.is_err());
    assert!(matches!(result, Err(BodoError::PluginError(_))));

    Ok(())
}
