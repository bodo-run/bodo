use clap::Parser;

use bodo::cli::Args;
use bodo::config::BodoConfig;
use bodo::errors::BodoError;
use bodo::graph::Graph;
use bodo::manager::GraphManager;
use bodo::plugin::Plugin;

// Test for CLI arguments parsing using clap's Parser trait.
#[test]
fn test_cli_parser() {
    let args = Args::parse_from([
        "bodo", "--debug", "-l", "mytask", "subtask", "--", "arg1", "arg2",
    ]);
    assert_eq!(args.task, Some("mytask".to_string()));
    assert_eq!(args.subtask, Some("subtask".to_string()));
    assert_eq!(args.args, vec!["arg1".to_string(), "arg2".to_string()]);
    assert!(args.debug);
    assert!(args.list);

    // Test default no-argument invocation.
    let default_args = Args::parse_from(["bodo"]);
    assert_eq!(default_args.task, None);
    assert_eq!(default_args.subtask, None);
    assert!(default_args.args.is_empty());
}

#[test]
fn test_bodo_config_generate_schema() {
    let schema = bodo::config::BodoConfig::generate_schema();
    serde_json::from_str::<serde_json::Value>(&schema).unwrap();
}

#[test]
fn test_graph_print_debug() {
    let graph = Graph::new();
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
fn test_graph_topological_sort_order() -> Result<(), BodoError> {
    let mut graph = Graph::new();
    let a = graph.add_node(bodo::graph::NodeKind::Task(bodo::graph::TaskData {
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
    let b = graph.add_node(bodo::graph::NodeKind::Task(bodo::graph::TaskData {
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
    graph.add_edge(a, b).unwrap();
    let sorted = graph.topological_sort()?;
    assert_eq!(sorted.len(), 2);
    assert!(sorted[0] == a && sorted[1] == b);
    Ok(())
}

#[test]
fn test_plugin_manager_run_lifecycle_default_config() -> Result<(), BodoError> {
    let mut pm = bodo::plugin::PluginManager::new();
    struct DummyPlugin;
    impl Plugin for DummyPlugin {
        fn name(&self) -> &'static str {
            "Dummy"
        }
        fn priority(&self) -> i32 {
            0
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
    pm.register(Box::new(DummyPlugin));
    let mut graph = Graph::new();
    pm.run_lifecycle(&mut graph, None)?;
    Ok(())
}

#[test]
fn test_bodo_error_from_yaml_error() {
    let yaml_err = serde_yaml::from_str::<serde_yaml::Value>(":").unwrap_err();
    let be: BodoError = yaml_err.into();
    match be {
        BodoError::YamlError(ref msg) => assert!(!msg.to_string().is_empty()),
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
fn test_cli_args_default() {
    let args = Args::parse_from(["bodo"]);
    assert!(!args.debug);
}

#[test]
fn test_get_task_config_nonexistent_task() {
    let manager = GraphManager::new();
    let result = manager.get_task_config("nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_apply_task_arguments() {
    let mut manager = GraphManager::new();
    let config = BodoConfig::default();
    manager.build_graph(config).unwrap();
    // Take the arguments from the task_config.arguments
    let result = manager.apply_task_arguments("nonexistent", &["arg".to_string()]);
    assert!(result.is_err());
}

#[test]
fn test_cli_args_parse_debug() {
    let args = Args::parse_from(["bodo", "--debug"]);
    assert!(args.debug);
}
