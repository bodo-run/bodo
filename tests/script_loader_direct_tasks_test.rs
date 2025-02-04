use bodo::config::BodoConfig;
use bodo::script_loader::ScriptLoader;

#[test]
fn test_build_graph_with_direct_tasks() {
    let config_yaml = r#"
tasks:
  task_direct:
    command: echo "Direct task"
    description: "A task defined directly in config"
"#;
    let config: BodoConfig = serde_yaml::from_str(config_yaml).unwrap();
    let mut loader = ScriptLoader::new();
    let graph = loader.build_graph(config).unwrap();
    assert!(graph.task_registry.contains_key("task_direct"));
}
