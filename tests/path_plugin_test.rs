use std::collections::HashMap;

use bodo::graph::{Graph, NodeKind, TaskData};
use bodo::plugin::{Plugin, PluginConfig};
use bodo::plugins::path_plugin::PathPlugin;

#[test]
fn test_path_plugin_on_init() {
    let mut plugin = PathPlugin::new();
    let mut config_options = serde_json::Map::new();
    config_options.insert(
        "default_paths".to_string(),
        serde_json::Value::Array(vec![serde_json::Value::String(
            "/usr/local/bin".to_string(),
        )]),
    );
    config_options.insert("preserve_path".to_string(), serde_json::Value::Bool(false));
    let config = PluginConfig {
        options: Some(config_options),
        ..Default::default()
    };
    let result = plugin.on_init(&config);
    assert!(result.is_ok());
    assert_eq!(
        plugin.get_default_paths(),
        &vec!["/usr/local/bin".to_string()]
    );
    assert!(!plugin.get_preserve_path());
}

#[test]
fn test_path_plugin_on_graph_build() {
    let mut plugin = PathPlugin::new();
    plugin.set_default_paths(vec!["/usr/bin".to_string()]);
    plugin.set_preserve_path(false);
    let mut graph = Graph::new();
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "test_task".to_string(),
        description: None,
        command: None,
        working_dir: Some("/home/user".to_string()),
        env: HashMap::new(),
        exec_paths: vec!["/custom/bin".to_string()],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
    }));

    let result = plugin.on_graph_build(&mut graph);
    assert!(result.is_ok());

    if let NodeKind::Task(task_data) = &graph.nodes[task_id as usize].kind {
        let path = task_data.env.get("PATH").expect("PATH not set");
        assert_eq!(path, "/home/user:/usr/bin:/custom/bin");
    } else {
        panic!("Expected Task node");
    }
}
