use std::collections::HashMap;

use bodo::{
    errors::BodoError,
    graph::{Graph, NodeKind, TaskData},
    plugin::{Plugin, PluginConfig, PluginManager},
    plugins::{concurrent_plugin::ConcurrentPlugin, execution_plugin::ExecutionPlugin},
    Result,
};
use serde_json::json;

fn make_graph_with_concurrent_tasks(
    tasks: Vec<(String, String)>,
    fail_fast: bool,
    max_concurrent: Option<usize>,
) -> Graph {
    let mut graph = Graph::new();
    let mut child_ids = Vec::new();

    // Create task nodes
    for (name, command) in tasks {
        let task = NodeKind::Task(TaskData {
            name: name.clone(),
            description: Some(format!("Test task {}", name)),
            command: Some(command),
            working_dir: None,
            is_default: false,
            script_name: Some("Test".to_string()),
            env: HashMap::new(),
        });
        let id = graph.add_node(task);
        child_ids.push(id);
    }

    // Add concurrency metadata to the first task
    if !child_ids.is_empty() {
        let first_task = &mut graph.nodes[child_ids[0] as usize];
        first_task.metadata.insert(
            "concurrently".to_string(),
            json!({
                "children": &child_ids[1..],
                "fail_fast": fail_fast,
                "max_concurrent": max_concurrent,
            })
            .to_string(),
        );
    }

    graph
}

#[tokio::test]
async fn test_concurrent_graph_construction() -> Result<()> {
    let failing = (
        "failing_task".to_string(),
        "sh -c 'sleep 1 && exit 1'".to_string(),
    );
    let long_running = (
        "long_task".to_string(),
        "sh -c 'sleep 5 && echo Long task finished'".to_string(),
    );

    let graph = make_graph_with_concurrent_tasks(vec![failing, long_running], true, None);

    assert_eq!(graph.nodes.len(), 2); // 2 tasks
    assert_eq!(graph.edges.len(), 0); // No edges yet

    let failing_node = graph.nodes.iter().find(|n| match &n.kind {
        NodeKind::Task(t) => t.name == "failing_task",
        _ => false,
    });
    assert!(failing_node.is_some());

    let long_running_node = graph.nodes.iter().find(|n| match &n.kind {
        NodeKind::Task(t) => t.name == "long_task",
        _ => false,
    });
    assert!(long_running_node.is_some());

    Ok(())
}

#[tokio::test]
async fn test_concurrent_plugin_transformation() -> Result<()> {
    let mut plugin = ConcurrentPlugin::new();
    let mut graph = make_graph_with_concurrent_tasks(
        vec![
            ("task1".to_string(), "echo task1".to_string()),
            ("task2".to_string(), "echo task2".to_string()),
        ],
        true,
        Some(2),
    );

    plugin.on_graph_build(&mut graph).await?;

    // After plugin transformation:
    // - Original 2 task nodes remain
    // - 1 new concurrent group node is added
    // - 1 edge from task1 to concurrent group
    assert_eq!(graph.nodes.len(), 3);
    assert_eq!(graph.edges.len(), 1);

    // Verify the concurrent group was created correctly
    let group_node = graph
        .nodes
        .iter()
        .find(|n| matches!(n.kind, NodeKind::ConcurrentGroup(_)));
    assert!(group_node.is_some());

    Ok(())
}
