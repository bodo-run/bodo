use bodo::config::BodoConfig;
use bodo::script_loader::ScriptLoader;

#[test]
fn test_build_graph_with_valid_tasks() {
    let config_yaml = r#"
default_task:
  command: echo "Default"
  description: Default task

tasks:
  task1:
    command: echo "Task1"
  task2:
    command: echo "Task2"
"#;
    let config: BodoConfig = serde_yaml::from_str(config_yaml).unwrap();
    let mut loader = ScriptLoader::new();
    let graph = loader.build_graph(config).unwrap();
    // Check that default and other tasks are registered.
    assert!(graph.task_registry.contains_key("default"));
    assert!(graph.task_registry.contains_key("task1"));
    assert!(graph.task_registry.contains_key("task2"));
}
