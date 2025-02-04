use bodo::errors::BodoError;
use bodo::graph::{CommandData, NodeKind, TaskData};
use bodo::manager::GraphManager;

#[test]
fn test_get_task_config_non_task_node() {
    let mut manager = GraphManager::new();
    // Add a Command node instead of a Task node.
    let node_id = manager.graph.add_node(NodeKind::Command(CommandData {
        raw_command: "echo".to_string(),
        description: None,
        working_dir: None,
        env: Default::default(),
        watch: None,
    }));
    manager
        .graph
        .task_registry
        .insert("command_task".to_string(), node_id);
    let result = manager.get_task_config("command_task");
    assert!(matches!(result, Err(BodoError::PluginError(_))));
}

#[test]
fn test_apply_task_arguments_non_task_node() {
    let mut manager = GraphManager::new();
    // Add a Command node and register it as a task.
    let node_id = manager
        .graph
        .add_node(NodeKind::Command(bodo::graph::CommandData {
            raw_command: "echo Hello".to_string(),
            description: Some("A command".to_string()),
            working_dir: None,
            env: Default::default(),
            watch: None,
        }));
    manager
        .graph
        .task_registry
        .insert("non_task".to_string(), node_id);
    let result = manager.apply_task_arguments("non_task", &["arg"]);
    assert!(result.is_err());
}
