use std::collections::HashMap;

use bodo::{
    graph::{Graph, NodeKind, TaskData},
    plugin::{Plugin, PluginConfig, PluginManager},
    plugins::print_list_plugin::PrintListPlugin,
    Result,
};

#[tokio::test]
async fn test_print_list_plugin() -> Result<()> {
    let mut graph = Graph::new();

    // Add root task
    let root_task = TaskData {
        name: "root".to_string(),
        description: Some("Root task".to_string()),
        command: Some("echo root".to_string()),
        working_dir: None,
        is_default: false,
        script_id: "root_script".to_string(),
        script_display_name: "Root Script".to_string(),
        env: HashMap::new(),
    };
    let root_id = graph.add_node(NodeKind::Task(root_task));

    // Add script task
    let script_task = TaskData {
        name: "script".to_string(),
        description: Some("Script task".to_string()),
        command: Some("echo script".to_string()),
        working_dir: None,
        is_default: false,
        script_id: "test_script".to_string(),
        script_display_name: "Test Script".to_string(),
        env: HashMap::new(),
    };
    let script_id = graph.add_node(NodeKind::Task(script_task));

    // Add edge from root to script
    let _ = graph.add_edge(root_id, script_id);

    // Setup plugins
    let mut manager = PluginManager::new();
    manager.register(Box::new(PrintListPlugin));

    // Run plugins to process metadata
    manager
        .run_lifecycle(
            &mut graph,
            Some(PluginConfig {
                fail_fast: false,
                watch: false,
                list: true,
                options: None,
            }),
        )
        .await?;

    Ok(())
}
