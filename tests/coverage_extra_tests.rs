use bodo::{manager::GraphManager, plugin::PluginConfig, BodoConfig, BodoError, Graph};
use std::env;

#[test]
fn test_bodo_config_generate_schema() {
    let schema = BodoConfig::generate_schema();
    assert!(schema.contains("BodoConfig"));
}

#[test]
fn test_graph_print_debug() {
    let mut graph = Graph::new();
    // Just call print_debug to ensure it executes without panic.
    graph.print_debug();
}

#[test]
fn test_graph_detect_cycle_none() {
    let mut graph = Graph::new();
    let _ = graph.add_node(bodo::graph::NodeKind::Task(bodo::graph::TaskData {
        name: "a".to_string(),
        description: None,
        command: Some("echo a".to_string()),
        working_dir: None,
        env: Default::default(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
    }));
    assert!(graph.detect_cycle().is_none());
}

#[test]
fn test_graph_detect_cycle_some() {
    let mut graph = Graph::new();
    let id1 = graph.add_node(bodo::graph::NodeKind::Task(bodo::graph::TaskData {
        name: "a".to_string(),
        description: None,
        command: Some("echo a".to_string()),
        working_dir: None,
        env: Default::default(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
    }));
    let id2 = graph.add_node(bodo::graph::NodeKind::Task(bodo::graph::TaskData {
        name: "b".to_string(),
        description: None,
        command: Some("echo b".to_string()),
        working_dir: None,
        env: Default::default(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
    }));
    graph.add_edge(id1, id2).unwrap();
    graph.add_edge(id2, id1).unwrap();
    let cycle = graph.detect_cycle();
    assert!(cycle.is_some());
}

#[test]
fn test_graph_topological_sort_order_call() {
    let mut graph = Graph::new();
    let id1 = graph.add_node(bodo::graph::NodeKind::Task(bodo::graph::TaskData {
        name: "A".to_string(),
        description: None,
        command: Some("echo A".to_string()),
        working_dir: None,
        env: Default::default(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
    }));
    let id2 = graph.add_node(bodo::graph::NodeKind::Task(bodo::graph::TaskData {
        name: "B".to_string(),
        description: None,
        command: Some("echo B".to_string()),
        working_dir: None,
        env: Default::default(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
    }));
    graph.add_edge(id1, id2).unwrap();
    let sorted = graph.topological_sort().unwrap();
    assert_eq!(sorted, vec![id1, id2]);
}

#[test]
fn test_manager_register_and_run_lifecycle() {
    struct DummyPlugin;
    impl bodo::plugin::Plugin for DummyPlugin {
        fn name(&self) -> &'static str {
            "Dummy"
        }
        fn priority(&self) -> i32 {
            0
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn on_run(&mut self, _node_id: usize, _graph: &mut Graph) -> Result<(), BodoError> {
            Ok(())
        }
    }

    let mut manager = GraphManager::new();
    manager.register_plugin(Box::new(DummyPlugin));
    let config = BodoConfig::default();
    manager.build_graph(config).unwrap();
    let result = manager.run_plugins(Some(PluginConfig::default()));
    assert!(result.is_ok());
}

#[test]
fn test_bodo_error_from_validation_errors() {
    use validator::ValidationErrors;
    let ve = ValidationErrors::new();
    let be: BodoError = ve.into();
    match be {
        BodoError::ValidationError(_) => {}
        _ => panic!("Expected ValidationError"),
    }
}

#[test]
fn test_bodo_error_from_serde_yaml_error() {
    let yaml_err = serde_yaml::from_str::<serde_yaml::Value>(":").unwrap_err();
    let be: BodoError = yaml_err.into();
    match be {
        BodoError::YamlError(ref msg) => assert!(!msg.is_empty()),
        _ => panic!("Expected YamlError"),
    }
}

#[test]
fn test_bodo_error_from_serde_json_error() {
    let json_err = serde_json::from_str::<serde_json::Value>(":").unwrap_err();
    let be: BodoError = json_err.into();
    match be {
        BodoError::SerdeError(_) => {}
        _ => panic!("Expected SerdeError"),
    }
}

#[test]
fn test_cli_args_default() {
    let args = bodo::cli::Args::parse_from(["bodo"]);
    assert!(!args.debug);
}

#[test]
fn test_path_plugin_default() {
    let pp = bodo::plugins::path_plugin::PathPlugin::new();
    let _ = pp.get_default_paths();
    let _ = pp.get_preserve_path();
}

#[test]
fn test_env_plugin_on_init_with_empty_options() {
    let mut plugin = bodo::plugins::env_plugin::EnvPlugin::new();
    let config = PluginConfig {
        options: None,
        ..Default::default()
    };
    let res = plugin.on_init(&config);
    assert!(res.is_ok());
    assert!(plugin.global_env.is_none());
}

#[test]
fn test_watch_plugin_find_base_directory_empty_pattern() {
    // When given an empty pattern, it should default to "."
    let base = bodo::plugins::watch_plugin::WatchPlugin::find_base_directory("");
    assert_eq!(base, Some(std::path::PathBuf::from(".")));
}

#[test]
fn test_watch_plugin_ignore_no_matches() {
    use globset::{Glob, GlobSetBuilder};
    let mut gbuilder = GlobSetBuilder::new();
    gbuilder.add(Glob::new("*.rs").unwrap());
    let glob_set = gbuilder.build().unwrap();
    let watch_entry = bodo::plugins::watch_plugin::WatchEntry {
        task_name: "dummy".to_string(),
        glob_set,
        ignore_set: None,
        directories_to_watch: std::collections::HashSet::new(),
        debounce_ms: 500,
    };
    // Pass an empty list, expecting empty result.
    let plugin = bodo::plugins::watch_plugin::WatchPlugin::new(false, false);
    let matches = plugin.filter_changed_paths(&vec![], &watch_entry);
    assert!(matches.is_empty());
}

#[test]
fn test_env_plugin_on_graph_build_no_env() {
    let mut plugin = bodo::plugins::env_plugin::EnvPlugin::new();
    plugin.global_env = None;
    let mut graph = Graph::new();
    let task_id = graph.add_node(bodo::graph::NodeKind::Task(bodo::graph::TaskData {
        name: "dummy".to_string(),
        description: None,
        command: Some("echo dummy".to_string()),
        working_dir: None,
        env: std::collections::HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    }));
    let res = plugin.on_graph_build(&mut graph);
    assert!(res.is_ok());
    if let bodo::graph::NodeKind::Task(task_data) = &graph.nodes[task_id as usize].kind {
        // Without global_env, the env map should remain empty
        assert!(task_data.env.get("TEST").is_none());
    }
}

#[test]
fn test_plugin_config_default_values() {
    let pc = PluginConfig::default();
    assert!(!pc.fail_fast);
    assert!(!pc.watch);
    assert!(!pc.list);
    assert!(pc.options.is_none());
}
