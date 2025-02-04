use bodo::config::BodoConfig;
use bodo::script_loader::ScriptLoader;

#[test]
fn test_build_graph_with_concurrently() {
    let config_yaml = r#"
default_task:
  command: echo "Default task"
  description: "Runs by default"
  concurrently:
    - task: subtask
    - command: echo "Command in concurrently"
tasks:
  subtask:
    command: echo "Subtask"
"#;
    let config: BodoConfig = serde_yaml::from_str(config_yaml).unwrap();
    let mut loader = ScriptLoader::new();
    let graph = loader.build_graph(config).unwrap();
    // Check that default and subtask tasks are registered.
    assert!(graph.task_registry.contains_key("default"));
    assert!(graph.task_registry.contains_key("subtask"));
}
