use std::collections::HashMap;

use bodo::{
    graph::{Graph, NodeKind, TaskData},
    plugin::Plugin,
    plugins::print_list_plugin::PrintListPlugin,
    Result,
};

#[tokio::test]
async fn test_print_list_plugin() -> Result<()> {
    let mut graph = Graph::new();

    // Add a root task
    let _root_id = graph.add_node(NodeKind::Task(TaskData {
        name: "root".to_string(),
        description: Some("Root task".to_string()),
        command: Some("echo root".to_string()),
        working_dir: None,
        is_default: false,
        script_name: None,
        env: HashMap::new(),
    }));

    // Add a task from a script
    let _script_id = graph.add_node(NodeKind::Task(TaskData {
        name: "script_task".to_string(),
        description: Some("Task from script".to_string()),
        command: Some("echo script".to_string()),
        working_dir: None,
        is_default: false,
        script_name: Some("test_script".to_string()),
        env: HashMap::new(),
    }));

    let mut plugin = PrintListPlugin;
    plugin.on_graph_build(&mut graph).await?;

    Ok(())
}
