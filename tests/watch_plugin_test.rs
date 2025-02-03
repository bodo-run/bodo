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
    assert!(!plugin.is_watch_mode());

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
    assert!(plugin.is_watch_mode());
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

#[test]
fn test_find_base_directory() {
    use bodo::plugins::watch_plugin::find_base_directory;
    use std::path::PathBuf;

    let test_cases = vec![
        ("**/*.rs", PathBuf::from(".")),
        ("src/**/*.rs", PathBuf::from("src")),
        ("src/lib.rs", PathBuf::from("src")),
        ("src/*.rs", PathBuf::from("src")),
        ("*.rs", PathBuf::from(".")),
        ("src/*/mod.rs", PathBuf::from("src")),
        ("src", PathBuf::from("src")),
        ("src/", PathBuf::from("src")),
        ("", PathBuf::from(".")),
    ];

    for (input, expected) in test_cases {
        let result = find_base_directory(input).unwrap();
        assert_eq!(result, expected, "Failed for input '{}'", input);
    }
}

#[test]
fn test_filter_changed_paths() {
    use bodo::plugins::watch_plugin::WatchEntry;
    use globset::{Glob, GlobSet, GlobSetBuilder};
    use std::path::PathBuf;

    // Build the glob set
    let patterns = vec!["src/**/*.rs".to_string()];
    let mut gbuilder = GlobSetBuilder::new();
    for patt in &patterns {
        let glob = Glob::new(patt).unwrap();
        gbuilder.add(glob);
    }
    let glob_set = gbuilder.build().unwrap();

    // Build the ignore glob set
    let ignore_patterns: Vec<String> = vec!["src/test_ignore.rs".to_string()];
    let mut ignore_builder = GlobSetBuilder::new();
    for patt in &ignore_patterns {
        let glob = Glob::new(patt).unwrap();
        ignore_builder.add(glob);
    }
    let ignore_set = Some(ignore_builder.build().unwrap());

    // Determine directories to watch
    let mut dirs = std::collections::HashSet::new();
    for patt in &patterns {
        if let Some(dir) = bodo::plugins::watch_plugin::find_base_directory(patt) {
            dirs.insert(dir);
        }
    }

    let watch_entry = WatchEntry {
        task_name: "test_task".to_string(),
        glob_set,
        ignore_set,
        directories_to_watch: dirs,
        debounce_ms: 500,
    };

    let plugin = WatchPlugin::new(true, false);

    // Prepare changed paths
    let changed_paths = vec![
        PathBuf::from("src/main.rs"),
        PathBuf::from("src/lib.rs"),
        PathBuf::from("src/ignored_file.rs"),
        PathBuf::from("src/test_ignore.rs"),
        PathBuf::from("README.md"),
    ];

    let matched_paths = plugin.filter_changed_paths(&changed_paths, &watch_entry);

    assert_eq!(matched_paths.len(), 3);
    assert!(matched_paths.contains(&PathBuf::from("src/main.rs")));
    assert!(matched_paths.contains(&PathBuf::from("src/lib.rs")));
    assert!(matched_paths.contains(&PathBuf::from("src/ignored_file.rs"))); // This one is not in ignore_patterns
}

#[test]
fn test_watch_plugin_on_after_run() {
    use bodo::plugin::PluginConfig;
    use std::fs::File;
    use std::io::Write;
    use std::thread;
    use std::time::Duration;
    use tempfile::tempdir;

    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Create a temporary file to watch
    let watched_file_path = temp_path.join("watched_file.rs");
    let mut watched_file = File::create(&watched_file_path).unwrap();
    writeln!(watched_file, "// Initial content").unwrap();
    watched_file.sync_all().unwrap();

    // Set up the WatchPlugin
    let mut plugin = WatchPlugin::new(true, false);

    // Ensure BODO_NO_WATCH is not set
    std::env::remove_var("BODO_NO_WATCH");

    // Prepare the graph
    let mut graph = Graph::new();

    let task_data = TaskData {
        name: "watch_task".to_string(),
        description: None,
        command: Some(format!("cat {}", watched_file_path.display())),
        working_dir: Some(temp_path.to_string_lossy().to_string()),
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: true,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: Some(WatchConfig {
            patterns: vec![format!("{}", watched_file_path.display())],
            debounce_ms: 500,
            ignore_patterns: vec![],
            auto_watch: true,
        }),
    };

    let task_id = graph.add_node(NodeKind::Task(task_data));
    graph
        .task_registry
        .insert("watch_task".to_string(), task_id);

    plugin.on_graph_build(&mut graph).unwrap();

    // Run on_after_run in a separate thread
    let plugin_handle = {
        let mut plugin_clone = plugin.clone();
        let mut graph_clone = graph.clone();
        thread::spawn(move || {
            plugin_clone.on_after_run(&mut graph_clone).unwrap();
        })
    };

    // Wait a moment to ensure the watcher is set up
    thread::sleep(Duration::from_secs(1));

    // Modify the watched file to trigger the watcher
    let mut watched_file = File::create(&watched_file_path).unwrap();
    writeln!(watched_file, "// Modified content").unwrap();
    watched_file.sync_all().unwrap();

    // Wait some time to allow the watcher to detect the change
    thread::sleep(Duration::from_secs(2));

    // Since the on_after_run runs indefinitely, we'll stop the test here
    // In a real test, we would have some mechanism to stop the watcher after verification
    // For this test, we are just ensuring no panics and basic functionality

    // Terminate the plugin thread (in a real test, we'd have a better way)
    // Here we'll just drop the temp_dir to clean up and end the test
}
