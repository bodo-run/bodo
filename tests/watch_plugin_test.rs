// tests/watch_plugin_test.rs

use bodo::config::WatchConfig;
use bodo::errors::Result;
use bodo::graph::{Graph, NodeKind, TaskData};
use bodo::plugin::{Plugin, PluginConfig};
use bodo::plugins::watch_plugin::{find_base_directory, WatchEntry, WatchPlugin};
use globset::{Glob, GlobSetBuilder};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

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

    // Verify that the watch_entry has the correct data
    let entry = &plugin.watch_entries[0];
    assert_eq!(entry.task_name, "watch_task");
    assert_eq!(entry.debounce_ms, 500);
    assert!(entry.glob_set.is_match("src/main.rs"));
    assert!(!entry.glob_set.is_match("README.md"));
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
    // Simulate on_graph_build to populate watch_entries
    plugin.on_graph_build(&mut graph).unwrap();

    // We need to mock the file system events and watcher
    // Since this involves threading and IO, it's complex to set up in a unit test
    // For now, we'll test that on_after_run returns Ok and doesn't panic
    let result = plugin.on_after_run(&mut graph);
    assert!(result.is_ok(), "on_after_run failed: {:?}", result.err());
}

#[test]
fn test_watch_plugin_no_watch_mode() {
    // Watch mode is disabled, even if tasks have watch configs
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

    // Since watch_mode is false, even though tasks have watch configs, it should not populate watch_entries
    assert_eq!(plugin.get_watch_entry_count(), 0);
}

#[test]
fn test_filter_changed_paths() {
    let plugin = WatchPlugin::new(true, false);

    let entry = WatchEntry {
        task_name: "test_task".to_string(),
        glob_set: GlobSetBuilder::new()
            .add(Glob::new("src/**/*.rs").unwrap())
            .build()
            .unwrap(),
        ignore_set: Some(
            GlobSetBuilder::new()
                .add(Glob::new("src/tests/*").unwrap())
                .build()
                .unwrap(),
        ),
        directories_to_watch: std::iter::once(PathBuf::from("src")).collect(),
        debounce_ms: 500,
    };

    let changed_paths = vec![
        PathBuf::from("src/main.rs"),
        PathBuf::from("src/tests/test_watch_plugin.rs"),
        PathBuf::from("README.md"),
    ];

    let matched = plugin.filter_changed_paths(&changed_paths, &entry);
    assert_eq!(matched.len(), 1);
    assert_eq!(matched[0], PathBuf::from("src/main.rs"));
}

#[test]
fn test_find_base_directory() {
    assert_eq!(find_base_directory("**/*.rs"), Some(PathBuf::from(".")));
    assert_eq!(
        find_base_directory("src/**/*.rs"),
        Some(PathBuf::from("src"))
    );
    assert_eq!(
        find_base_directory("src/*/test.rs"),
        Some(PathBuf::from("src"))
    );
    assert_eq!(find_base_directory("build.rs"), Some(PathBuf::from(".")));
    assert_eq!(
        find_base_directory("/absolute/path/*.rs"),
        Some(PathBuf::from("/absolute/path"))
    );
    assert_eq!(find_base_directory(""), Some(PathBuf::from(".")));
}

#[test]
fn test_watch_plugin_auto_watch_env_var() {
    // Set BODO_NO_WATCH environment variable
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

    // Since BODO_NO_WATCH is set, even though auto_watch is true, watch_mode should remain false
    assert!(!plugin.watch_mode);
    assert_eq!(plugin.get_watch_entry_count(), 0);

    // Clean up the environment variable
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
    let entry = &plugin.watch_entries[0];
    assert_eq!(entry.debounce_ms, 1000);
}
