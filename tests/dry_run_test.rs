use std::collections::HashMap;

use bodo::graph::{Graph, Node, NodeKind, TaskData};
use bodo::plugin::{DryRun, DryRunReport, PluginConfig, SimulatedAction};
use bodo::plugins::execution_plugin::ExecutionPlugin;
use bodo::BodoError;

#[test]
fn test_simulated_action_creation() {
    let mut details = HashMap::new();
    details.insert("command".to_string(), "echo hello".to_string());

    let action = SimulatedAction {
        action_type: "command".to_string(),
        description: "Execute command".to_string(),
        details,
        node_id: Some(1),
    };

    assert_eq!(action.action_type, "command");
    assert_eq!(action.description, "Execute command");
    assert_eq!(
        action.details.get("command"),
        Some(&"echo hello".to_string())
    );
    assert_eq!(action.node_id, Some(1));
}

#[test]
fn test_dry_run_report_creation() {
    let action = SimulatedAction {
        action_type: "command".to_string(),
        description: "Execute command".to_string(),
        details: HashMap::new(),
        node_id: Some(1),
    };

    let mut metadata = HashMap::new();
    metadata.insert("total_actions".to_string(), "1".to_string());

    let report = DryRunReport {
        plugin_name: "TestPlugin".to_string(),
        simulated_actions: vec![action],
        dependencies: vec!["task1".to_string()],
        warnings: vec!["Warning message".to_string()],
        metadata,
    };

    assert_eq!(report.plugin_name, "TestPlugin");
    assert_eq!(report.simulated_actions.len(), 1);
    assert_eq!(report.dependencies, vec!["task1".to_string()]);
    assert_eq!(report.warnings, vec!["Warning message".to_string()]);
    assert_eq!(report.metadata.get("total_actions"), Some(&"1".to_string()));
}

#[allow(unused_mut)]
#[test]
fn test_execution_plugin_dry_run_no_task() {
    let plugin = ExecutionPlugin::new();
    let mut graph = Graph::new();
    let config = PluginConfig::default();

    let result = plugin.dry_run_simulate(&graph, &config);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), BodoError::PluginError(_)));
}

#[test]
fn test_execution_plugin_dry_run_simple_task() {
    let mut plugin = ExecutionPlugin::new();
    let mut graph = Graph::new();

    // Create a simple task
    let task_data = TaskData {
        name: "test_task".to_string(),
        command: Some("echo hello".to_string()),
        description: Some("Test task".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    };

    let node = Node {
        id: 0,
        kind: NodeKind::Task(task_data),
        metadata: HashMap::new(),
    };

    graph.nodes.push(node);
    graph.task_registry.insert("test_task".to_string(), 0);

    // Set the task name
    plugin.task_name = Some("test_task".to_string());

    let config = PluginConfig::default();
    let result = plugin.dry_run_simulate(&graph, &config);

    assert!(result.is_ok());
    let report = result.unwrap();

    assert_eq!(report.plugin_name, "ExecutionPlugin");
    assert_eq!(report.simulated_actions.len(), 1);
    assert_eq!(report.simulated_actions[0].action_type, "command");
    assert_eq!(
        report.simulated_actions[0].description,
        "Execute task 'test_task'"
    );
    assert_eq!(
        report.simulated_actions[0].details.get("command"),
        Some(&"echo hello".to_string())
    );
    assert_eq!(report.dependencies, vec!["test_task".to_string()]);
    assert!(report.warnings.is_empty());
}

#[test]
fn test_execution_plugin_dry_run_with_env_vars() {
    let mut plugin = ExecutionPlugin::new();
    let mut graph = Graph::new();

    // Create a task with environment variables
    let mut env = HashMap::new();
    env.insert("NAME".to_string(), "world".to_string());

    let task_data = TaskData {
        name: "test_task".to_string(),
        command: Some("echo hello $NAME".to_string()),
        description: Some("Test task".to_string()),
        working_dir: None,
        env,
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    };

    let node = Node {
        id: 0,
        kind: NodeKind::Task(task_data),
        metadata: HashMap::new(),
    };

    graph.nodes.push(node);
    graph.task_registry.insert("test_task".to_string(), 0);

    // Set the task name
    plugin.task_name = Some("test_task".to_string());

    let config = PluginConfig::default();
    let result = plugin.dry_run_simulate(&graph, &config);

    assert!(result.is_ok());
    let report = result.unwrap();

    assert_eq!(report.simulated_actions.len(), 1);
    assert_eq!(
        report.simulated_actions[0].details.get("command"),
        Some(&"echo hello world".to_string())
    );
}

#[test]
fn test_execution_plugin_dry_run_unresolved_env_vars() {
    let mut plugin = ExecutionPlugin::new();
    let mut graph = Graph::new();

    // Create a task with unresolved environment variables
    let task_data = TaskData {
        name: "test_task".to_string(),
        command: Some("echo hello $UNDEFINED_VAR".to_string()),
        description: Some("Test task".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    };

    let node = Node {
        id: 0,
        kind: NodeKind::Task(task_data),
        metadata: HashMap::new(),
    };

    graph.nodes.push(node);
    graph.task_registry.insert("test_task".to_string(), 0);

    // Set the task name
    plugin.task_name = Some("test_task".to_string());

    let config = PluginConfig::default();
    let result = plugin.dry_run_simulate(&graph, &config);

    assert!(result.is_ok());
    let report = result.unwrap();

    assert_eq!(report.simulated_actions.len(), 1);
    assert_eq!(
        report.simulated_actions[0].details.get("command"),
        Some(&"echo hello $UNDEFINED_VAR".to_string())
    );
    assert_eq!(report.warnings.len(), 1);
    assert!(report.warnings[0].contains("unresolved environment variables"));
}

#[test]
fn test_execution_plugin_dry_run_with_working_dir() {
    let mut plugin = ExecutionPlugin::new();
    let mut graph = Graph::new();

    // Create a task with working directory
    let task_data = TaskData {
        name: "test_task".to_string(),
        command: Some("echo hello".to_string()),
        description: Some("Test task".to_string()),
        working_dir: Some("/tmp".to_string()),
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    };

    let node = Node {
        id: 0,
        kind: NodeKind::Task(task_data),
        metadata: HashMap::new(),
    };

    graph.nodes.push(node);
    graph.task_registry.insert("test_task".to_string(), 0);

    // Set the task name
    plugin.task_name = Some("test_task".to_string());

    let config = PluginConfig::default();
    let result = plugin.dry_run_simulate(&graph, &config);

    assert!(result.is_ok());
    let report = result.unwrap();

    assert_eq!(report.simulated_actions.len(), 1);
    assert_eq!(
        report.simulated_actions[0].details.get("working_directory"),
        Some(&"/tmp".to_string())
    );
}

#[test]
fn test_execution_plugin_dry_run_task_not_found() {
    let mut plugin = ExecutionPlugin::new();
    let graph = Graph::new();

    // Set a non-existent task name
    plugin.task_name = Some("non_existent_task".to_string());

    let config = PluginConfig::default();
    let result = plugin.dry_run_simulate(&graph, &config);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), BodoError::TaskNotFound(_)));
}

#[test]
fn test_dry_run_report_serialization() {
    let action = SimulatedAction {
        action_type: "command".to_string(),
        description: "Execute command".to_string(),
        details: HashMap::new(),
        node_id: Some(1),
    };

    let report = DryRunReport {
        plugin_name: "TestPlugin".to_string(),
        simulated_actions: vec![action],
        dependencies: vec!["task1".to_string()],
        warnings: vec!["Warning".to_string()],
        metadata: HashMap::new(),
    };

    // Test JSON serialization
    let json = serde_json::to_string(&report).unwrap();
    let deserialized: DryRunReport = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.plugin_name, report.plugin_name);
    assert_eq!(
        deserialized.simulated_actions.len(),
        report.simulated_actions.len()
    );
    assert_eq!(deserialized.dependencies, report.dependencies);
    assert_eq!(deserialized.warnings, report.warnings);
}
