use bodo::config::WatchConfig;
use bodo::graph::{NodeKind, TaskData};
use bodo::plugin::Plugin;
use bodo::plugins::watch_plugin::WatchPlugin;
use bodo::Graph;
use std::collections::HashMap;

#[test]
fn test_watch_plugin_on_init_no_watch() {
    let mut plugin = WatchPlugin::new(false, false);
    let config = bodo::plugin::PluginConfig {
        watch: false,
        ..Default::default()
    };
    plugin.on_init(&config).unwrap();
    assert!(!plugin.watch_mode);
}

#[test]
fn test_watch_plugin_on_init_with_watch() {
    let mut plugin = WatchPlugin::new(false, false);
    let config = bodo::plugin::PluginConfig {
        watch: true,
        ..Default::default()
    };
    plugin.on_init(&config).unwrap();
    assert!(plugin.watch_mode);
}

#[test]
fn test_watch_plugin_on_graph_build_with_auto_watch_and_env_var_set() {
    let mut plugin = WatchPlugin::new(false, false);

    // Set BODO_NO_WATCH environment variable
    std::env::set_var("BODO_NO_WATCH", "1");

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

    let _node_id = graph.add_node(NodeKind::Task(task_data));

    plugin.on_graph_build(&mut graph).unwrap();

    // Ensure that watch_mode remains false due to BODO_NO_WATCH
    assert!(!plugin.watch_mode);

    // Unset the environment variable for other tests
    std::env::remove_var("BODO_NO_WATCH");
}

#[test]
fn test_watch_plugin_on_graph_build_with_auto_watch() {
    let mut plugin = WatchPlugin::new(false, false);

    // Ensure BODO_NO_WATCH is not set
    std::env::remove_var("BODO_NO_WATCH");

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

    let _node_id = graph.add_node(NodeKind::Task(task_data));

    plugin.on_graph_build(&mut graph).unwrap();

    // Ensure that watch_mode is now true due to auto_watch
    assert!(plugin.watch_mode);
}

#[test]
fn test_watch_plugin_no_auto_watch_no_watch_mode() {
    let mut plugin = WatchPlugin::new(false, false);

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
            auto_watch: false,
        }),
    };

    let _node_id = graph.add_node(NodeKind::Task(task_data));

    plugin.on_graph_build(&mut graph).unwrap();

    // Since watch_mode is false, watch entries should not be populated
    assert_eq!(plugin.get_watch_entry_count(), 0);
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

    let _node_id = graph.add_node(NodeKind::Task(task_data));
    plugin.on_graph_build(&mut graph).unwrap();

    // Ensure that the watch_entries were populated
    assert_eq!(plugin.get_watch_entry_count(), 1);
}
