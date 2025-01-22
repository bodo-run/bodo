use bodo::{
    graph::{Graph, NodeKind, TaskData},
    plugin::{PluginConfig, PluginManager},
    plugins::resolver_plugin::ResolverPlugin,
    Result,
};
use serde_json::json;
use std::collections::HashMap;

#[tokio::test]
async fn test_pre_deps_resolution() -> Result<()> {
    let mut graph = Graph::new();

    // Add tasks
    let build_id = graph.add_node(NodeKind::Task(TaskData {
        name: "build".into(),
        description: Some("Build the project".into()),
        command: Some("echo Building".into()),
        working_dir: None,
        env: HashMap::new(),
        is_default: false,
        script_name: None,
    }));

    let test_id = graph.add_node(NodeKind::Task(TaskData {
        name: "test".into(),
        description: Some("Run tests".into()),
        command: Some("echo Testing".into()),
        working_dir: None,
        env: HashMap::new(),
        is_default: false,
        script_name: None,
    }));

    // Register tasks
    graph.task_registry.insert("build".to_string(), build_id);
    graph.task_registry.insert("test".to_string(), test_id);

    // Add pre_deps metadata
    let node = &mut graph.nodes[test_id as usize];
    node.metadata
        .insert("pre_deps".to_string(), json!(["build"]).to_string());

    // Create plugin manager and run resolver
    let mut manager = PluginManager::new();
    manager.register(Box::new(ResolverPlugin));
    manager
        .run_lifecycle(&mut graph, &PluginConfig::default())
        .await?;

    // Verify edge was created
    assert_eq!(graph.edges.len(), 1);
    assert_eq!(graph.edges[0].from, build_id);
    assert_eq!(graph.edges[0].to, test_id);

    Ok(())
}

#[tokio::test]
async fn test_post_deps_resolution() -> Result<()> {
    let mut graph = Graph::new();

    // Add tasks
    let build_id = graph.add_node(NodeKind::Task(TaskData {
        name: "build".into(),
        description: Some("Build the project".into()),
        command: Some("echo Building".into()),
        working_dir: None,
        env: HashMap::new(),
        is_default: false,
        script_name: None,
    }));

    let test_id = graph.add_node(NodeKind::Task(TaskData {
        name: "test".into(),
        description: Some("Run tests".into()),
        command: Some("echo Testing".into()),
        working_dir: None,
        env: HashMap::new(),
        is_default: false,
        script_name: None,
    }));

    // Register tasks
    graph.task_registry.insert("build".to_string(), build_id);
    graph.task_registry.insert("test".to_string(), test_id);

    // Add post_deps metadata
    let node = &mut graph.nodes[build_id as usize];
    node.metadata
        .insert("post_deps".to_string(), json!(["test"]).to_string());

    // Create plugin manager and run resolver
    let mut manager = PluginManager::new();
    manager.register(Box::new(ResolverPlugin));
    manager
        .run_lifecycle(&mut graph, &PluginConfig::default())
        .await?;

    // Verify edge was created
    assert_eq!(graph.edges.len(), 1);
    assert_eq!(graph.edges[0].from, build_id);
    assert_eq!(graph.edges[0].to, test_id);

    Ok(())
}

#[tokio::test]
async fn test_missing_dependency() -> Result<()> {
    let mut graph = Graph::new();

    // Add task
    let test_id = graph.add_node(NodeKind::Task(TaskData {
        name: "test".into(),
        description: Some("Run tests".into()),
        command: Some("echo Testing".into()),
        working_dir: None,
        env: HashMap::new(),
        is_default: false,
        script_name: None,
    }));

    // Register task
    graph.task_registry.insert("test".to_string(), test_id);

    // Add pre_deps metadata with missing dependency
    let node = &mut graph.nodes[test_id as usize];
    node.metadata.insert(
        "pre_deps".to_string(),
        json!(["build"]).to_string(), // build task doesn't exist
    );

    // Create plugin manager and run resolver
    let mut manager = PluginManager::new();
    manager.register(Box::new(ResolverPlugin));

    // Should fail with dependency not found error
    let result = manager
        .run_lifecycle(&mut graph, &PluginConfig::default())
        .await;
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("Dependency build not found"));
    }

    Ok(())
}
