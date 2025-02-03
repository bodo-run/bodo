// tests/path_plugin_test.rs

use std::collections::HashMap;

use bodo::graph::{Graph, NodeKind, TaskData};
use bodo::plugin::Plugin;
use bodo::plugins::path_plugin::PathPlugin;

#[test]
fn test_path_plugin() {
    let mut plugin = PathPlugin::new();

    let config = bodo::plugin::PluginConfig {
        options: Some(
            serde_json::json!({
                "default_paths": ["/usr/local/bin"],
                "preserve_path": false
            })
            .as_object()
            .cloned()
            .unwrap(),
        ),
        ..Default::default()
    };

    plugin.on_init(&config).unwrap();

    let mut graph = Graph::new();

    let task_data = TaskData {
        name: "test_task".to_string(),
        description: None,
        command: Some("echo $PATH".to_string()),
        working_dir: Some("/home/user".to_string()),
        env: HashMap::new(),
        exec_paths: vec!["/custom/bin".to_string()],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    };

    let node_id = graph.add_node(NodeKind::Task(task_data));

    // Apply the plugin
    plugin.on_graph_build(&mut graph).unwrap();

    // Check that the PATH environment variable is set correctly
    let node = &graph.nodes[node_id as usize];
    if let NodeKind::Task(task_data) = &node.kind {
        let path_env = task_data.env.get("PATH").expect("PATH not set");
        let expected_path = "/home/user:/usr/local/bin:/custom/bin".to_string();
        assert_eq!(path_env, &expected_path);
    } else {
        panic!("Expected Task node");
    }
}
