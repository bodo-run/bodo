use bodo::config::BodoConfig;
use bodo::script_loader::ScriptLoader;

#[test]
fn test_build_graph_no_root_script() {
    // Create a configuration with tasks defined directly (no root_script).
    let config_yaml = r#"
default_task:
  command: echo "Default without root"
  description: "Default task without root_script"

tasks:
  direct:
    command: echo "Direct task"
    description: "Task defined directly in config"
"#;
    let config: BodoConfig = serde_yaml::from_str(config_yaml).unwrap();
    let mut loader = ScriptLoader::new();
    let graph = loader.build_graph(config).unwrap();

    // Tasks should be registered using their task names.
    assert!(graph.task_registry.contains_key("direct"));
    assert!(graph.task_registry.contains_key("default"));
}
