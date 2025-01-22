use bodo::{
    graph::{Graph, NodeKind, TaskData},
    plugin::{PluginConfig, PluginManager},
    plugins::{
        concurrency_plugin::ConcurrencyPlugin,
        env_plugin::EnvPlugin,
        execution_plugin::{execute_graph, ExecutionPlugin},
        resolver_plugin::ResolverPlugin,
        watch_plugin::WatchPlugin,
    },
    Result,
};
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    // Create a simple graph with two tasks
    let mut graph = Graph::new();

    // Add a "build" task
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "build".into(),
        description: Some("Build the project".into()),
        command: Some("echo Building".into()),
        working_dir: None,
        env: HashMap::new(),
        is_default: false,
        script_name: None,
    }));

    // Add a "test" task
    let test_id = graph.add_node(NodeKind::Task(TaskData {
        name: "test".into(),
        description: Some("Run tests".into()),
        command: Some("echo Testing".into()),
        working_dir: None,
        env: HashMap::new(),
        is_default: false,
        script_name: None,
    }));

    // Register tasks in registry
    graph.task_registry.insert("build".to_string(), task_id);
    graph.task_registry.insert("test".to_string(), test_id);

    // Add dependency metadata to the test task
    {
        let node = &mut graph.nodes[test_id as usize];
        node.metadata
            .insert("pre_deps".to_string(), json!(["build"]).to_string());
    }

    // Create and configure plugins
    let mut manager = PluginManager::new();
    manager.register(Box::new(ResolverPlugin));
    manager.register(Box::new(ConcurrencyPlugin));
    manager.register(Box::new(EnvPlugin::new()));
    manager.register(Box::new(WatchPlugin::new()));
    manager.register(Box::new(ExecutionPlugin));

    // Configure global environment variables
    let plugin_config = PluginConfig {
        options: Some(
            json!({
                "env": {
                    "RUST_LOG": "info"
                }
            })
            .as_object()
            .cloned()
            .unwrap(),
        ),
    };

    // Run plugin lifecycle
    manager.run_lifecycle(&mut graph, &plugin_config).await?;

    // Execute the graph
    execute_graph(&mut manager, &mut graph).await?;

    Ok(())
}
