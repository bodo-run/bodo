use bodo::errors::BodoError;
use bodo::graph::{Graph, NodeKind, TaskData};
use bodo::plugin::Plugin;
use bodo::plugins::timeout_plugin::TimeoutPlugin;
use std::collections::HashMap;

#[test]
fn test_timeout_plugin_on_graph_build_sets_timeout() -> Result<(), BodoError> {
    let mut plugin = TimeoutPlugin::new();
    let mut graph = Graph::new();
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "test_task".to_string(),
        description: Some("A test task".to_string()),
        command: Some("sleep 5".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    }));
    {
        let node = &mut graph.nodes[task_id as usize];
        node.metadata
            .insert("timeout".to_string(), "30s".to_string());
    }
    plugin.on_graph_build(&mut graph)?;
    let node = &graph.nodes[task_id as usize];
    assert_eq!(
        node.metadata.get("timeout_seconds"),
        Some(&"30".to_string())
    );
    Ok(())
}
