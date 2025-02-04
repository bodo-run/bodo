use std::collections::HashMap;
use std::io;
use tempfile::tempdir;

#[test]
fn test_cli_get_task_name_error() {
    use bodo::cli::{get_task_name, Args};
    use bodo::manager::GraphManager;
    let args = Args::parse_from(["bodo"]);
    let manager = GraphManager::new();
    let result = get_task_name(&args, &manager);
    assert!(result.is_err());
}

#[test]
fn test_generate_schema() {
    use bodo::config::BodoConfig;
    let schema = BodoConfig::generate_schema();
    assert!(schema.contains("\"title\": \"BodoConfig\""));
}

#[test]
fn test_errors_conversions() {
    use bodo::errors::BodoError;
    let io_err = io::Error::new(io::ErrorKind::Other, "simulated error");
    let converted: BodoError = io_err.into();
    assert!(format!("{}", converted).contains("simulated error"));

    let serde_json_err = serde_json::from_str::<serde_json::Value>(":").unwrap_err();
    let converted: BodoError = serde_json_err.into();
    assert!(format!("{}", converted).to_lowercase().contains("expected"));
}

#[test]
fn test_graph_functions() {
    use bodo::graph::{Graph, NodeKind, TaskData};
    let mut graph = Graph::new();
    let node_a = graph.add_node(NodeKind::Task(TaskData {
        name: "A".to_string(),
        description: None,
        command: Some("echo A".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    }));
    let node_b = graph.add_node(NodeKind::Task(TaskData {
        name: "B".to_string(),
        description: None,
        command: Some("echo B".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    }));
    // Add valid edge, then test print_debug and topological_sort
    graph.add_edge(node_a, node_b).unwrap();
    graph.print_debug();
    let name = graph.node_name(node_a as usize);
    assert!(!name.is_empty());
    let sorted = graph.topological_sort().unwrap();
    assert_eq!(sorted.len(), 2);
    let cycle = graph.detect_cycle();
    assert!(cycle.is_none());
}

#[test]
fn test_manager_functions() {
    use bodo::config::BodoConfig;
    use bodo::manager::GraphManager;
    let mut manager = GraphManager::new();
    let config = BodoConfig {
        tasks: Default::default(),
        env: Default::default(),
        exec_paths: vec![],
        root_script: None,
        scripts_dirs: None,
        default_task: None,
    };
    manager.build_graph(config).unwrap();
    assert!(!manager.task_exists("nonexistent"));
    let result = manager.get_task_config("nonexistent");
    assert!(result.is_err());
    let res = manager.apply_task_arguments("nonexistent", &["arg"]);
    assert!(res.is_err());
    // Call initialize even if it's a dummy config.
    let _ = manager.initialize();
}

#[test]
fn test_dummy_plugin_methods() {
    use bodo::plugin::{Plugin, PluginConfig};
    struct Dummy;
    impl Plugin for Dummy {
        fn name(&self) -> &'static str {
            "Dummy"
        }
        fn priority(&self) -> i32 {
            0
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn on_run(
            &mut self,
            _node_id: usize,
            _graph: &mut bodo::graph::Graph,
        ) -> Result<(), bodo::errors::BodoError> {
            Ok(())
        }
    }
    let mut dummy = Dummy;
    let config = PluginConfig::default();
    assert!(dummy.on_init(&config).is_ok());
    let mut graph = bodo::graph::Graph::new();
    assert!(dummy.on_graph_build(&mut graph).is_ok());
    assert!(dummy.on_after_run(&mut graph).is_ok());
    assert!(dummy.on_run(0, &mut graph).is_ok());
}

#[test]
fn test_path_plugin_test_build_path() {
    use bodo::plugins::path_plugin::PathPlugin;
    let mut plugin = PathPlugin::new();
    plugin.set_default_paths(vec!["/default".to_string()]);
    plugin.set_preserve_path(false);
    let result = plugin.test_build_path(Some(&"/work".to_string()), &[String::from("/exec")]);
    assert_eq!(result, "/work:/default:/exec");
}

#[test]
fn test_process_color_functions() {
    use bodo::process::{color_line, parse_color};
    let color = parse_color("red");
    assert_eq!(color, Some(colored::Color::Red));
    let line = color_line("Test", &Some("red".to_string()), "line content", false);
    assert!(line.contains("Test"));
}

#[test]
fn test_script_loader_merge_functions() {
    use bodo::script_loader::ScriptLoader;
    let global = HashMap::from([("A".to_string(), "1".to_string())]);
    let script = HashMap::from([("B".to_string(), "2".to_string())]);
    let task = HashMap::from([("C".to_string(), "3".to_string())]);
    let merged = ScriptLoader::merge_envs(&global, &script, &task);
    assert_eq!(merged.get("A"), Some(&"1".to_string()));
    let merged_paths = ScriptLoader::merge_exec_paths(
        &vec!["/global".to_string()],
        &vec!["/global".to_string(), "/script".to_string()],
        &vec!["/script".to_string(), "/task".to_string()],
    );
    assert_eq!(
        merged_paths,
        vec![
            "/global".to_string(),
            "/script".to_string(),
            "/task".to_string()
        ]
    );
}

#[test]
fn test_timeout_plugin_parse_timeout() {
    use bodo::plugins::timeout_plugin::parse_timeout;
    let seconds = parse_timeout("30s").unwrap();
    assert_eq!(seconds, 30);
    let invalid = parse_timeout("invalid");
    assert!(invalid.is_err());
}

#[test]
fn test_cli_args_parse() {
    use bodo::cli::Args;
    let args = Args::parse_from(["bodo", "--debug"]);
    assert!(args.debug);
}

#[test]
fn test_plugin_config_defaults() {
    use bodo::plugin::PluginConfig;
    let pc = PluginConfig::default();
    assert!(!pc.fail_fast);
    assert!(!pc.watch);
    assert!(!pc.list);
    assert!(pc.options.is_none());
}

#[test]
fn test_watch_plugin_find_base_directory() {
    use bodo::plugins::watch_plugin::WatchPlugin;
    use std::path::PathBuf;
    let cases = vec![
        ("**/*.rs", PathBuf::from(".")),
        ("src/**/*.rs", PathBuf::from("src")),
        ("src/lib.rs", PathBuf::from("src")),
        ("*.rs", PathBuf::from(".")),
        ("", PathBuf::from(".")),
    ];
    for (input, expected) in cases {
        let res = WatchPlugin::find_base_directory(input).unwrap();
        assert_eq!(res, expected, "Failed for input '{}'", input);
    }
}

#[test]
fn test_watch_plugin_filter_changed_paths() {
    use bodo::plugins::watch_plugin::{WatchEntry, WatchPlugin};
    use globset::{Glob, GlobSetBuilder};
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    let temp_dir = tempdir().unwrap();
    let cwd = env::current_dir().unwrap();
    let base_dir = temp_dir.path().join("src");
    fs::create_dir_all(&base_dir).unwrap();
    let file1 = base_dir.join("main.rs");
    let file2 = base_dir.join("lib.rs");
    let file3 = base_dir.join("ignore.rs");
    fs::write(&file1, "test").unwrap();
    fs::write(&file2, "test").unwrap();
    fs::write(&file3, "test").unwrap();

    let patterns = vec!["src/**/*.rs".to_string()];
    let mut gbuilder = GlobSetBuilder::new();
    for patt in &patterns {
        let glob = Glob::new(patt).unwrap();
        gbuilder.add(glob);
    }
    let glob_set = gbuilder.build().unwrap();

    let ignore_patterns = vec!["src/ignore.rs".to_string()];
    let mut ibuilder = GlobSetBuilder::new();
    for patt in &ignore_patterns {
        let glob = Glob::new(patt).unwrap();
        ibuilder.add(glob);
    }
    let ignore_set = Some(ibuilder.build().unwrap());

    let mut dirs = std::collections::HashSet::new();
    dirs.insert(temp_dir.path().join("src"));

    let entry = WatchEntry {
        task_name: "dummy".to_string(),
        glob_set,
        ignore_set,
        directories_to_watch: dirs,
        debounce_ms: 500,
    };

    let plugin = WatchPlugin::new(true, false);

    // Change working directory to temp_dir for relative path calculation
    let original = env::current_dir().unwrap();
    env::set_current_dir(temp_dir.path()).unwrap();

    let changed = vec![
        PathBuf::from("src/main.rs"),
        PathBuf::from("src/lib.rs"),
        PathBuf::from("src/ignore.rs"),
        PathBuf::from("README.md"),
    ];
    let filtered = plugin.filter_changed_paths(&changed, &entry);
    // Expect 2 files: main.rs and lib.rs
    assert_eq!(filtered.len(), 2);

    env::set_current_dir(original).unwrap();
}

#[test]
fn test_watch_plugin_on_init_switch() {
    use bodo::plugin::PluginConfig;
    use bodo::plugins::watch_plugin::WatchPlugin;
    let mut plugin = WatchPlugin::new(false, false);
    let config = PluginConfig {
        watch: true,
        ..Default::default()
    };
    plugin.on_init(&config).unwrap();
    assert!(plugin.is_watch_mode());
}

#[test]
fn test_watch_plugin_on_init_no_switch() {
    use bodo::plugin::PluginConfig;
    use bodo::plugins::watch_plugin::WatchPlugin;
    let mut plugin = WatchPlugin::new(false, false);
    let config = PluginConfig {
        watch: false,
        ..Default::default()
    };
    plugin.on_init(&config).unwrap();
    assert!(!plugin.is_watch_mode());
}

#[test]
fn test_cli_args_default_values() {
    use bodo::cli::Args;
    let args = Args::parse_from(["bodo"]);
    assert!(args.args.is_empty());
    assert!(!args.debug);
    assert!(!args.list);
}

#[test]
fn test_timeout_plugin_on_graph_build_with_timeout() {
    use bodo::graph::{Graph, NodeKind, TaskData};
    use bodo::plugins::timeout_plugin::TimeoutPlugin;
    use std::collections::HashMap;
    let mut plugin = TimeoutPlugin::new();
    let mut graph = Graph::new();
    let node = graph.add_node(NodeKind::Task(TaskData {
        name: "timeout_test".to_string(),
        description: None,
        command: Some("sleep 5".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    }));
    graph.nodes[node as usize]
        .metadata
        .insert("timeout".to_string(), "2s".to_string());
    let res = plugin.on_graph_build(&mut graph);
    assert!(res.is_ok());
    let meta = &graph.nodes[node as usize].metadata;
    assert!(meta.contains_key("timeout_seconds"));
}

#[test]
fn test_plugin_manager_sort_and_run_lifecycle() {
    use bodo::plugin::{Plugin, PluginConfig, PluginManager};
    struct LowPriority;
    impl Plugin for LowPriority {
        fn name(&self) -> &'static str {
            "Low"
        }
        fn priority(&self) -> i32 {
            10
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
    struct HighPriority;
    impl Plugin for HighPriority {
        fn name(&self) -> &'static str {
            "High"
        }
        fn priority(&self) -> i32 {
            100
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
    let mut manager = PluginManager::new();
    manager.register(Box::new(LowPriority));
    manager.register(Box::new(HighPriority));
    manager.sort_plugins();
    let plugins = manager.get_plugins();
    // First plugin should have higher priority.
    assert!(plugins[0].priority() >= plugins[1].priority());
    let mut graph = bodo::graph::Graph::new();
    let config = PluginConfig::default();
    assert!(manager.run_lifecycle(&mut graph, Some(config)).is_ok());
}
