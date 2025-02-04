// tests/plugins_test.rs

use bodo::errors::{BodoError, Result};
use bodo::graph::{Graph, Node, NodeKind, TaskData};
use bodo::plugin::{Plugin, PluginConfig};
use bodo::plugins::execution_plugin::ExecutionPlugin;
use std::collections::HashMap;

// Re-implement expand_env_vars function locally for testing
fn expand_env_vars(cmd: &str, env: &HashMap<String, String>) -> String {
    let mut result = String::new();
    let mut chars = cmd.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' {
            // Handle environment variable
            if let Some(peek) = chars.peek() {
                if *peek == '$' {
                    result.push('$');
                    chars.next();
                } else if *peek == '{' {
                    // ${VAR}
                    chars.next(); // Consume '{'
                    let mut var_name = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch == '}' {
                            chars.next(); // Consume '}'
                            break;
                        } else {
                            var_name.push(ch);
                            chars.next();
                        }
                    }
                    if let Some(var_value) = env.get(&var_name) {
                        result.push_str(var_value);
                    } else {
                        // If the variable is not in env, keep it as is
                        result.push_str(&format!("${{{}}}", var_name));
                    }
                } else {
                    // $VAR
                    let mut var_name = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch.is_alphanumeric() || ch == '_' {
                            var_name.push(ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    if let Some(var_value) = env.get(&var_name) {
                        result.push_str(var_value);
                    } else {
                        // If the variable is not in env, keep it as is
                        result.push_str(&format!("${}", var_name));
                    }
                }
            } else {
                result.push('$');
            }
        } else {
            result.push(c);
        }
    }
    result
}

#[test]
fn test_execution_plugin_on_init() -> Result<()> {
    let mut plugin = ExecutionPlugin::new();
    let mut options = serde_json::Map::new();
    options.insert(
        "task".to_string(),
        serde_json::Value::String("test_task".to_string()),
    );
    let config = PluginConfig {
        fail_fast: true,
        watch: false,
        list: false,
        options: Some(options),
    };
    plugin.on_init(&config)?;
    assert_eq!(plugin.task_name.as_deref(), Some("test_task"));
    Ok(())
}

#[test]
fn test_execution_plugin_on_after_run_no_task_specified() -> Result<()> {
    let mut plugin = ExecutionPlugin::new();
    let mut graph = Graph::new();
    let result = plugin.on_after_run(&mut graph);
    assert!(matches!(result, Err(BodoError::PluginError(_))));
    Ok(())
}

#[test]
fn test_execution_plugin_on_after_run() -> Result<()> {
    let mut plugin = ExecutionPlugin::new();
    plugin.task_name = Some("test_task".to_string());
    let mut graph = Graph::new();
    let node_id = graph.add_node(NodeKind::Task(TaskData {
        name: "test_task".to_string(),
        description: None,
        command: Some("echo 'Hello World'".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: true,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
    }));
    graph.task_registry.insert("test_task".to_string(), node_id);
    let result = plugin.on_after_run(&mut graph);
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_get_prefix_settings() {
    let plugin = ExecutionPlugin::new();
    let mut node = Node {
        id: 0,
        kind: NodeKind::Task(TaskData {
            name: "test_task".to_string(),
            description: None,
            command: None,
            working_dir: None,
            env: HashMap::new(),
            exec_paths: vec![],
            arguments: vec![],
            is_default: false,
            script_id: "".to_string(),
            script_display_name: "".to_string(),
            watch: None,
        }),
        metadata: HashMap::new(),
    };

    node.metadata
        .insert("prefix_enabled".to_string(), "true".to_string());
    node.metadata
        .insert("prefix_label".to_string(), "test_label".to_string());
    node.metadata
        .insert("prefix_color".to_string(), "green".to_string());

    let (enabled, label, color) = plugin.get_prefix_settings(&node);
    assert!(enabled);
    assert_eq!(label, Some("test_label".to_string()));
    assert_eq!(color, Some("green".to_string()));
}

// Rest of the tests...
