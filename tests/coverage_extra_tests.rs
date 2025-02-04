use bodo::cli::Args;
use bodo::BodoError;
use bodo::Graph;
use bodo::Plugin; // Import Plugin trait to have access to plugin methods.
use clap::Parser;

fn dummy_function() -> Result<(), BodoError> {
    Ok(())
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
    let in_degree = [0; 2];
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
    assert_eq!(sorted, vec![a, b]);
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
    let args = Args::try_parse_from(["bodo"]).unwrap();
    assert!(!args.debug);
}
