// tests/watch_plugin_test.rs

use bodo::config::WatchConfig;
use bodo::graph::{Graph, NodeKind, TaskData};
use bodo::plugin::{Plugin, PluginConfig};
use bodo::plugins::watch_plugin::WatchPlugin;
use std::collections::HashMap;

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
    let mut plugin = WatchPlugin::new(true, false);
    let mut graph = Graph::new();

    let task_data = TaskData {
        name: "watch_task".to_string(),
        description: None,
        command: Some("echo 'Watching files'".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: Some(WatchConfig {
            patterns: vec!["src/**/*.rs".to_string()],
            debounce_ms: 500,
            ignore_patterns: vec![],
            auto_watch: true,
        }),
    };

    let node_id = graph.add_node(NodeKind::Task(task_data));
    plugin.on_graph_build(&mut graph).unwrap();

    // Ensure that the watch_entries were populated
    assert_eq!(plugin.get_watch_entry_count(), 1);

    // Since we cannot access private fields, we cannot verify internal data
}

#[test]
fn test_watch_plugin_on_after_run_executes() {
    let mut plugin = WatchPlugin::new(true, false);
    let mut graph = Graph::new();

    let task_data = TaskData {
        name: "watch_task".to_string(),
        description: None,
        command: Some("echo 'Watching files'".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: Some(WatchConfig {
            patterns: vec!["src/**/*.rs".to_string()],
            debounce_ms: 500,
            ignore_patterns: vec![],
            auto_watch: true,
        }),
    };

    graph.add_node(NodeKind::Task(task_data));
    plugin.on_graph_build(&mut graph).unwrap();

    let result = plugin.on_after_run(&mut graph);
    assert!(result.is_ok(), "on_after_run failed: {:?}", result.err());
}

#[test]
fn test_watch_plugin_no_watch_mode() {
    let mut plugin = WatchPlugin::new(false, false);
    let mut graph = Graph::new();

    let task_data = TaskData {
        name: "task_with_watch".to_string(),
        description: None,
        command: Some("echo 'Should not watch'".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: Some(WatchConfig {
            patterns: vec!["src/**/*.rs".to_string()],
            debounce_ms: 500,
            ignore_patterns: vec![],
            auto_watch: false,
        }),
    };

    graph.add_node(NodeKind::Task(task_data));
    plugin.on_graph_build(&mut graph).unwrap();

    assert_eq!(plugin.get_watch_entry_count(), 0);
}

#[test]
fn test_watch_plugin_auto_watch_env_var() {
    std::env::set_var("BODO_NO_WATCH", "1");

    let mut plugin = WatchPlugin::new(false, false);
    let mut graph = Graph::new();

    let task_data = TaskData {
        name: "task_with_auto_watch".to_string(),
        description: None,
        command: Some("echo 'Auto watch'".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: Some(WatchConfig {
            patterns: vec!["src/**/*.rs".to_string()],
            debounce_ms: 500,
            ignore_patterns: vec![],
            auto_watch: true,
        }),
    };

    graph.add_node(NodeKind::Task(task_data));
    plugin.on_graph_build(&mut graph).unwrap();

    // Since BODO_NO_WATCH is set, watch_entries should be empty
    assert_eq!(plugin.get_watch_entry_count(), 0);

    std::env::remove_var("BODO_NO_WATCH");
}

#[test]
fn test_watch_plugin_debounce_setting() {
    let mut plugin = WatchPlugin::new(true, false);
    let mut graph = Graph::new();

    let task_data = TaskData {
        name: "debounce_task".to_string(),
        description: None,
        command: Some("echo 'Debounce Test'".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: Some(WatchConfig {
            patterns: vec!["src/**/*".to_string()],
            debounce_ms: 1000,
            ignore_patterns: vec![],
            auto_watch: false,
        }),
    };

    graph.add_node(NodeKind::Task(task_data));
    plugin.on_graph_build(&mut graph).unwrap();

    assert_eq!(plugin.get_watch_entry_count(), 1);

    // Since we cannot access private fields, we cannot check debounce_ms directly
}
