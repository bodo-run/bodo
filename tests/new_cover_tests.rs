use bodo::config::{validate_task_name, BodoConfig, TaskArgument, TaskConfig, WatchConfig};
use bodo::graph::{CommandData, ConcurrentGroupData, Graph, Node, NodeKind, TaskData};
use bodo::plugins::concurrent_plugin::ConcurrentPlugin;
use bodo::plugins::execution_plugin::ExecutionPlugin;
use bodo::plugins::timeout_plugin::TimeoutPlugin;
use bodo::script_loader::ScriptLoader;
use std::collections::HashMap;

// Test task name validations.
#[test]
fn test_invalid_task_name_chars() {
    let invalid_names = ["invalid/task", "invalid..name", "invalid.name"];
    for &name in &invalid_names {
        assert!(validate_task_name(name).is_err());
    }
}

#[test]
fn test_valid_task_name() {
    let valid_names = ["build", "test_task", "deploy"];
    for &name in &valid_names {
        assert!(validate_task_name(name).is_ok());
    }
}

#[test]
fn test_reserved_task_name() {
    let reserved = [
        "watch",
        "default_task",
        "pre_deps",
        "post_deps",
        "concurrently",
    ];
    for &name in &reserved {
        assert!(validate_task_name(name).is_err());
    }
}

// Test ScriptLoader with a valid YAML configuration.
#[test]
fn test_script_loader_valid_config() {
    let yaml = r#"
default_task:
  command: "echo default"
  description: "default task"
tasks:
  build:
    command: "cargo build"
    description: "build project"
  test:
    command: "cargo test"
"#;
    let config: BodoConfig = serde_yaml::from_str(yaml).expect("Should parse YAML");
    let mut loader = ScriptLoader::new();
    let graph = loader.build_graph(config).expect("Should build graph");
    // Check that task_registry contains expected keys.
    assert!(graph.task_registry.contains_key("default"));
    assert!(graph.task_registry.contains_key("build"));
    assert!(graph.task_registry.contains_key("test"));
}

// Duplicate task keys cannot be simulated with a HashMap, so we check that when inserting the same key twice,
// the later value overrides the former.
#[test]
fn test_script_loader_duplicate_task_override() {
    let task_config = TaskConfig {
        command: Some("echo 'first'".to_string()),
        description: Some("first task".to_string()),
        ..Default::default()
    };
    let task_config2 = TaskConfig {
        command: Some("echo 'second'".to_string()),
        description: Some("second task".to_string()),
        ..Default::default()
    };
    let mut tasks_map = HashMap::new();
    tasks_map.insert("duplicate".to_string(), task_config);
    // Insert same key; this will override.
    tasks_map.insert("duplicate".to_string(), task_config2);
    let config = BodoConfig {
        root_script: Some("script.yaml".to_string()),
        tasks: tasks_map,
        ..Default::default()
    };
    let mut loader = ScriptLoader::new();
    let graph = loader.build_graph(config).expect("Graph should build");
    // Only one task key exists.
    assert_eq!(graph.task_registry.len(), 1);
}

// Test merging of environment variables and exec paths.
#[test]
fn test_script_loader_merge_envs_and_exec_paths() {
    let global = HashMap::from([
        ("A".to_string(), "1".to_string()),
        ("B".to_string(), "2".to_string()),
    ]);
    let script = HashMap::from([
        ("B".to_string(), "script".to_string()),
        ("C".to_string(), "3".to_string()),
    ]);
    let task = HashMap::from([
        ("C".to_string(), "task".to_string()),
        ("D".to_string(), "4".to_string()),
    ]);
    let merged = ScriptLoader::merge_envs(&global, &script, &task);
    assert_eq!(merged.get("A"), Some(&"1".to_string()));
    assert_eq!(merged.get("B"), Some(&"script".to_string()));
    assert_eq!(merged.get("C"), Some(&"task".to_string()));
    assert_eq!(merged.get("D"), Some(&"4".to_string()));

    let global_paths = vec!["/global".to_string(), "/common".to_string()];
    let script_paths = vec!["/script".to_string(), "/common".to_string()];
    let task_paths = vec!["/task".to_string()];
    let merged_paths = ScriptLoader::merge_exec_paths(&global_paths, &script_paths, &task_paths);
    assert_eq!(
        merged_paths,
        vec![
            "/global".to_string(),
            "/common".to_string(),
            "/script".to_string(),
            "/task".to_string()
        ]
    );
}

// Test TimeoutPlugin parse functionality.
#[test]
fn test_timeout_plugin_parse_duration() {
    let secs = TimeoutPlugin::parse_timeout("45s").unwrap();
    assert_eq!(secs, 45);
    let mins = TimeoutPlugin::parse_timeout("2m").unwrap();
    assert_eq!(mins, 120);
}

#[test]
fn test_timeout_plugin_parse_invalid() {
    let result = TimeoutPlugin::parse_timeout("invalid");
    assert!(result.is_err());
}

// Test special cases for expand_env_vars in ExecutionPlugin.
#[test]
fn test_execution_plugin_expand_env_vars_special_cases() {
    let plugin = ExecutionPlugin::new();
    // $$ should return a single $
    let result = plugin.expand_env_vars("echo $$", &HashMap::new());
    assert_eq!(result, "echo $");
    // Character after $ not alphanumeric remains unchanged.
    let result = plugin.expand_env_vars("echo $%", &HashMap::new());
    assert_eq!(result, "echo $%");
}

// Test ConcurrentPlugin handling of invalid concurrently metadata.
#[test]
fn test_concurrent_plugin_handle_invalid_json() {
    let mut plugin = ConcurrentPlugin::new();
    let mut graph = Graph::new();
    let mut node = Node {
        id: 0,
        kind: NodeKind::Task(TaskData {
            name: "test".to_string(),
            description: None,
            command: Some("echo test".to_string()),
            working_dir: None,
            env: HashMap::new(),
            exec_paths: vec![],
            arguments: vec![],
            is_default: false,
            script_id: "script".to_string(),
            script_display_name: "script".to_string(),
            watch: None,
            pre_deps: vec![],
            post_deps: vec![],
            concurrently: vec![],
            concurrently_options: Default::default(),
        }),
        metadata: HashMap::new(),
    };
    // Insert invalid JSON for "concurrently"
    node.metadata
        .insert("concurrently".to_string(), "invalid json".to_string());
    graph.nodes.push(node);
    let result = plugin.on_graph_build(&mut graph);
    assert!(result.is_err());
}

// Test PrefixPlugin does not update nodes if "prefix_output" is not enabled.
#[test]
fn test_prefix_plugin_no_updates_when_no_prefix_output() {
    use bodo::plugins::prefix_plugin::PrefixPlugin;
    let mut plugin = PrefixPlugin::new();
    let mut graph = Graph::new();
    let node_id = graph.add_node(NodeKind::Task(TaskData {
        name: "noprefix".to_string(),
        description: None,
        command: Some("echo no prefix".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    }));
    let result = plugin.on_graph_build(&mut graph);
    assert!(result.is_ok());
    let node = &graph.nodes[node_id as usize];
    assert!(node.metadata.get("prefix_enabled").is_none());
    assert!(node.metadata.get("prefix_label").is_none());
    assert!(node.metadata.get("prefix_color").is_none());
}

// Additional test: Ensure that watch plugin configuration that does not enable auto_watch returns no entries.
#[test]
fn test_watch_plugin_no_auto_watch_no_entries() {
    use bodo::plugins::watch_plugin::WatchPlugin;
    let mut plugin = WatchPlugin::new(false, false);
    let mut graph = Graph::new();
    let task = bodo::graph::TaskData {
        name: "watch_task".to_string(),
        description: None,
        command: Some("echo 'Watching'".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: Some(WatchConfig {
            patterns: vec!["dummy.txt".to_string()],
            debounce_ms: 500,
            ignore_patterns: vec![],
            auto_watch: false,
        }),
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    };
    graph.nodes.push(Node {
        id: 0,
        kind: NodeKind::Task(task),
        metadata: HashMap::new(),
    });
    let res = plugin.on_graph_build(&mut graph);
    assert!(res.is_ok());
    assert_eq!(plugin.get_watch_entry_count(), 0);
}
