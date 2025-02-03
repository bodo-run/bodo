use bodo::config::WatchConfig;
use bodo::graph::{Graph, NodeKind, TaskData};
use bodo::plugin::{Plugin, PluginConfig};
use bodo::plugins::watch_plugin::WatchPlugin;
use std::collections::HashMap;

#[test]
fn test_watch_plugin_on_graph_build_auto_watch() {
    let mut plugin = WatchPlugin::new(false, false);
    let mut graph = Graph::new();
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "test_task".to_string(),
        description: None,
        command: Some("echo Hello".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: Some(WatchConfig {
            patterns: vec!["src/**/*.rs".to_string()],
            debounce_ms: 500,
            ignore_patterns: vec![],
            auto_watch: true,
        }),
    }));

    let config = PluginConfig {
        watch: false,
        ..Default::default()
    };

    let result = plugin.on_init(&config);
    assert!(result.is_ok());

    let result = plugin.on_graph_build(&mut graph);
    assert!(result.is_ok());

    assert!(plugin.watch_mode);
    assert_eq!(plugin.watch_entries.len(), 1);
}
