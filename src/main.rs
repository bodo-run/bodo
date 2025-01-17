use bodo::{
    graph::{Graph, NodeKind, TaskData},
    plugin::{PluginConfig, PluginManager},
    plugins::{
        concurrency_plugin::ConcurrencyPlugin,
        env_plugin::EnvPlugin,
        execution_plugin::{execute_graph, ExecutionPlugin},
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

    // Add concurrency metadata to the build task
    {
        let node = &mut graph.nodes[task_id as usize];
        node.metadata.insert(
            "concurrently".to_string(),
            json!({
                "children": [test_id],
                "fail_fast": true
            })
            .to_string(),
        );
    }

    // Create and configure plugins
    let mut manager = PluginManager::new();
    manager.register(Box::new(ConcurrencyPlugin));
    manager.register(Box::new(EnvPlugin::new()));
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
