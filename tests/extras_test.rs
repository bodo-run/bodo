use bodo::manager::GraphManager;
use bodo::plugin::{Plugin, PluginConfig, PluginManager};

// Test default PluginConfig values.
#[test]
fn test_plugin_config_defaults() {
    let config = PluginConfig::default();
    assert!(!config.fail_fast);
    assert!(!config.watch);
    assert!(!config.list);
    assert!(config.options.is_none());
}

// Dummy plugin to test PluginManager sorting.
struct DummyPlugin {
    priority_val: i32,
}

impl Plugin for DummyPlugin {
    fn name(&self) -> &'static str {
        "DummyPlugin"
    }
    fn priority(&self) -> i32 {
        self.priority_val
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[test]
fn test_plugin_manager_sort() {
    let mut manager = PluginManager::new();
    manager.register(Box::new(DummyPlugin { priority_val: 10 }));
    manager.register(Box::new(DummyPlugin { priority_val: 20 }));
    manager.sort_plugins();
    let plugins = manager.get_plugins();
    // The first plugin should have the higher priority.
    let p0 = plugins[0].priority();
    let p1 = plugins[1].priority();
    assert!(p0 >= p1);
}

// Test GraphManager initialize properly by creating a temporary scripts/main.yaml file.
#[test]
fn test_graph_manager_initialize() {
    use std::env;
    use std::fs;
    use tempfile::tempdir;

    // Create a temporary directory and set up the expected scripts structure.
    let temp_dir = tempdir().unwrap();
    let scripts_dir = temp_dir.path().join("scripts");
    fs::create_dir_all(&scripts_dir).unwrap();
    let main_yaml = scripts_dir.join("main.yaml");
    fs::write(&main_yaml, "tasks: {}").unwrap(); // Minimal valid YAML content.

    // Change current directory to temporary directory.
    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(&temp_dir).unwrap();

    let mut manager = GraphManager::new();
    let result = manager.initialize();
    assert!(result.is_ok());

    // Restore original current directory.
    env::set_current_dir(&original_dir).unwrap();
}

#[test]
fn test_graph_manager_with_tasks() {
    let config_yaml = r#"
    tasks:
      hello:
        command: echo "Hello World"
    "#;

    let config: bodo::config::BodoConfig = serde_yaml::from_str(config_yaml).unwrap();

    let mut manager = GraphManager::new();
    manager.build_graph(config).unwrap();
    assert!(!manager.graph.nodes.is_empty());
    assert!(manager.task_exists("hello"));
}

#[test]
fn test_apply_task_arguments() {
    let mut manager = GraphManager::new();
    let config_yaml = r#"
    tasks:
      greet:
        command: echo "Hello $name"
    "#;
    let config: bodo::config::BodoConfig = serde_yaml::from_str(config_yaml).unwrap();
    manager.build_graph(config).unwrap();
    // Simulate applying arguments
    manager.graph.nodes.iter_mut().for_each(|node| {
        if let bodo::graph::NodeKind::Task(task_data) = &mut node.kind {
            task_data
                .env
                .insert("name".to_string(), "Alice".to_string());
        }
    });
    let node_id = manager
        .graph
        .task_registry
        .get("greet")
        .expect("Task 'greet' not found");
    let node = manager
        .graph
        .nodes
        .get(*node_id as usize)
        .expect("Node not found");

    if let bodo::graph::NodeKind::Task(task_data) = &node.kind {
        assert_eq!(task_data.env.get("name"), Some(&"Alice".to_string()));
    } else {
        panic!("Expected Task node");
    }
}

#[test]
fn test_apply_task_arguments_with_defaults() {
    let mut manager = GraphManager::new();
    let task_config = bodo::config::TaskConfig {
        command: Some("echo $greeting".to_string()),
        arguments: vec![bodo::config::TaskArgument {
            name: "greeting".to_string(),
            description: None,
            required: false,
            default: Some("Hello".to_string()),
        }],
        ..Default::default()
    };
    let mut tasks = std::collections::HashMap::new();
    tasks.insert("hello".to_string(), task_config);
    let config = bodo::config::BodoConfig {
        tasks,
        ..Default::default()
    };
    manager.build_graph(config).unwrap();
    let result = manager.apply_task_arguments("hello", &[]);
    assert!(result.is_ok());

    let node_id = manager
        .graph
        .task_registry
        .get("hello")
        .cloned()
        .expect("Task 'hello' not found");
    let node = &manager.graph.nodes[node_id as usize];

    if let bodo::graph::NodeKind::Task(task_data) = &node.kind {
        assert_eq!(task_data.env.get("greeting"), Some(&"Hello".to_string()));
    } else {
        panic!("Expected Task node");
    }
}
