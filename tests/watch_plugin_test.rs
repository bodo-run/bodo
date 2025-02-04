use bodo::graph::{Graph, NodeKind, TaskData};
use bodo::plugins::watch_plugin::{WatchEntry, WatchPlugin};
use std::collections::HashMap;

#[test]
fn test_watch_plugin_on_init_no_watch() {
    let mut plugin = WatchPlugin::new(false, false);
    let config = bodo::plugin::PluginConfig {
        watch: false,
        ..Default::default()
    };
    plugin.on_init(&config).unwrap();
    assert!(!plugin.is_watch_mode());
}

#[test]
fn test_watch_plugin_on_init_with_watch() {
    let mut plugin = WatchPlugin::new(false, false);
    let config = bodo::plugin::PluginConfig {
        watch: true,
        ..Default::default()
    };
    plugin.on_init(&config).unwrap();
    assert!(plugin.is_watch_mode());
}

#[test]
fn test_watch_plugin_on_graph_build_with_auto_watch_and_env_var_set() {
    let mut plugin = WatchPlugin::new(false, false);

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
        watch: Some(bodo::config::WatchConfig {
            patterns: vec!["src/**/*.rs".to_string()],
            debounce_ms: 500,
            ignore_patterns: vec![],
            auto_watch: true,
        }),
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    };

    let _node_id = graph.add_node(NodeKind::Task(task_data));

    plugin.on_graph_build(&mut graph).unwrap();

    assert!(!plugin.is_watch_mode());

    std::env::remove_var("BODO_NO_WATCH");
}

#[test]
fn test_watch_plugin_on_graph_build_with_auto_watch() {
    let mut plugin = WatchPlugin::new(false, false);

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
        watch: Some(bodo::config::WatchConfig {
            patterns: vec!["src/**/*.rs".to_string()],
            debounce_ms: 500,
            ignore_patterns: vec![],
            auto_watch: true,
        }),
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    };

    let _node_id = graph.add_node(NodeKind::Task(task_data));

    plugin.on_graph_build(&mut graph).unwrap();

    assert!(plugin.is_watch_mode());
    assert_eq!(plugin.get_watch_entry_count(), 1);
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
        watch: Some(bodo::config::WatchConfig {
            patterns: vec!["src/**/*.rs".to_string()],
            debounce_ms: 500,
            ignore_patterns: vec![],
            auto_watch: false,
        }),
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    };

    let _node_id = graph.add_node(NodeKind::Task(task_data));

    plugin.on_graph_build(&mut graph).unwrap();

    assert_eq!(plugin.get_watch_entry_count(), 0);
}

#[test]
fn test_watch_plugin_on_graph_build_with_tasks() {
    let mut plugin = WatchPlugin::new(true, false);
    let mut graph = Graph::new();

    let task_data = TaskData {
        name: "watch_task".to_string(),
        description: None,
        command: Some(format!("cat {}", "dummy.txt")),
        working_dir: Some(".".to_string()),
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: true,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: Some(bodo::config::WatchConfig {
            patterns: vec!["dummy.txt".to_string()],
            debounce_ms: 500,
            ignore_patterns: vec![],
            auto_watch: true,
        }),
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    };

    let _node_id = graph.add_node(NodeKind::Task(task_data));
    plugin.on_graph_build(&mut graph).unwrap();

    assert_eq!(plugin.get_watch_entry_count(), 1);
}
