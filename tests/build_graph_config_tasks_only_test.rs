use bodo::config::BodoConfig;
use bodo::script_loader::ScriptLoader;

#[test]
fn test_build_graph_config_tasks_only() {
    let config_yaml = r#"
tasks:
  task1:
    command: echo "Task1"
"#;
    let config: BodoConfig = serde_yaml::from_str(config_yaml).unwrap();
    let mut loader = ScriptLoader::new();
    let graph = loader.build_graph(config).unwrap();
    assert!(!graph.nodes.is_empty(), "Graph should not be empty");
    assert!(
        graph.task_registry.contains_key("task1"),
        "Task 'task1' should be registered"
    );
}
