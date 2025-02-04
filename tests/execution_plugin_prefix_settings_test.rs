use bodo::graph::{Node, NodeKind, TaskData};
use bodo::plugins::execution_plugin::ExecutionPlugin;
use std::collections::HashMap;

#[test]
fn test_get_prefix_settings() {
    let plugin = ExecutionPlugin::new();
    let node = Node {
        id: 0,
        kind: NodeKind::Task(TaskData {
            name: "test".to_string(),
            description: None,
            command: Some("echo test".to_string()),
            working_dir: None,
            env: HashMap::new(),
            exec_paths: vec![],
            arguments: vec![],
            is_default: false,
            script_id: "script".to_string(),
            script_display_name: "".to_string(),
            watch: None,
            pre_deps: vec![],
            post_deps: vec![],
            concurrently: vec![],
            concurrently_options: Default::default(),
        }),
        metadata: {
            let mut m = HashMap::new();
            m.insert("prefix_enabled".to_string(), "true".to_string());
            m.insert("prefix_label".to_string(), "TEST".to_string());
            m.insert("prefix_color".to_string(), "blue".to_string());
            m
        },
    };
    let (enabled, label, color) = plugin.get_prefix_settings(&node);
    assert!(enabled);
    assert_eq!(label, Some("TEST".to_string()));
    assert_eq!(color, Some("blue".to_string()));
}
