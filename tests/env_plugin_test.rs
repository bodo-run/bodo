use std::collections::HashMap;

use bodo::graph::{Graph, NodeKind, TaskData};
use bodo::plugin::{Plugin, PluginConfig};
use bodo::plugins::env_plugin::EnvPlugin;

#[test]
fn test_env_plugin_on_init() {
    let mut plugin = EnvPlugin::new();
    let mut options = serde_json::Map::new();
    if let serde_json::Value::Object(ref mut obj) = serde_json::json!({"TEST_ENV": "test_value"}) {
        options.insert("env".to_string(), serde_json::Value::Object(obj.clone()));
    }
    let config = PluginConfig {
        options: Some(options),
        ..Default::default()
    };
    let result = plugin.on_init(&config);
    assert!(result.is_ok());
    assert!(plugin.global_env.is_some());
    assert_eq!(
        plugin.global_env.as_ref().unwrap().get("TEST_ENV"),
        Some(&"test_value".to_string())
    );
}

#[test]
fn test_env_plugin_on_graph_build() {
    let mut plugin = EnvPlugin::new();
    plugin.global_env = Some(HashMap::from([(
        "GLOBAL_ENV".to_string(),
        "value".to_string(),
    )]));

    let mut graph = Graph::new();
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "test_task".to_string(),
        description: None,
        command: Some("echo $GLOBAL_ENV".to_string()),
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
    }));
    let result = plugin.on_graph_build(&mut graph);
    assert!(result.is_ok());

    if let NodeKind::Task(task_data) = &graph.nodes[task_id as usize].kind {
        assert_eq!(task_data.env.get("GLOBAL_ENV"), Some(&"value".to_string()));
    } else {
        panic!("Expected Task node");
    }
}

#[test]
fn test_env_plugin_on_init_no_options() {
    let mut plugin = EnvPlugin::new();
    let config = PluginConfig {
        options: None,
        ..Default::default()
    };

    let result = plugin.on_init(&config);
    assert!(result.is_ok());
    assert!(plugin.global_env.is_none());
}

#[test]
fn test_env_plugin_on_init_invalid_options() {
    let mut plugin = EnvPlugin::new();
    let mut options = serde_json::Map::new();

    options.insert(
        "env".to_string(),
        serde_json::Value::String("invalid".to_string()),
    );

    let config = PluginConfig {
        options: Some(options),
        ..Default::default()
    };

    let result = plugin.on_init(&config);
    assert!(result.is_ok());
    assert!(plugin.global_env.is_none());
}

#[test]
fn test_env_plugin_on_graph_build_no_global_env() {
    let mut plugin = EnvPlugin::new();
    plugin.global_env = None;

    let mut graph = Graph::new();

    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "test_task".to_string(),
        description: None,
        command: Some("echo $GLOBAL_ENV".to_string()),
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
    }));
    let result = plugin.on_graph_build(&mut graph);
    assert!(result.is_ok());

    if let NodeKind::Task(task_data) = &graph.nodes[task_id as usize].kind {
        assert!(!task_data.env.contains_key("GLOBAL_ENV"));
    } else {
        panic!("Expected Task node");
    }
}
