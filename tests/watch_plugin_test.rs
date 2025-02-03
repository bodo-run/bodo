// tests/watch_plugin_test.rs

use bodo::graph::Graph;
use bodo::plugin::{Plugin, PluginConfig};
use bodo::plugins::watch_plugin::WatchPlugin;

#[test]
fn test_watch_plugin_on_init_with_config() {
    let mut plugin = WatchPlugin::new(true, false);
    let config = PluginConfig {
        watch: true,
        ..Default::default()
    };

    let result = plugin.on_init(&config);
    assert!(
        result.is_ok(),
        "WatchPlugin on_init failed with error: {:?}",
        result.err()
    );
}

#[test]
fn test_watch_plugin_on_graph_build_with_tasks() {
    use bodo::graph::{NodeKind, TaskData};
    use std::collections::HashMap;

    let mut plugin = WatchPlugin::new(true, false);
    let mut graph = Graph::new();

    let task_data = TaskData {
        name: "watch_task".to_string(),
        description: None,
        command: Some("echo 'Watching files'".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "".to_string(),
        watch: Some(bodo::config::WatchConfig {
            patterns: vec!["src/**/*.rs".to_string()],
            debounce_ms: 500,
            ignore_patterns: vec![],
            auto_watch: true,
        }),
    };

    let node_id = graph.add_node(NodeKind::Task(task_data));
    let result = plugin.on_graph_build(&mut graph);
    assert!(
        result.is_ok(),
        "WatchPlugin on_graph_build failed with error: {:?}",
        result.err()
    );

    // Since WatchPlugin doesn't implement functionality, we can't test more
}

#[test]
fn test_watch_plugin_on_after_run_executes() {
    let mut plugin = WatchPlugin::new(true, false);
    let mut graph = Graph::new();

    let result = plugin.on_after_run(&mut graph);
    assert!(
        result.is_ok(),
        "WatchPlugin on_after_run failed with error: {:?}",
        result.err()
    );
}
