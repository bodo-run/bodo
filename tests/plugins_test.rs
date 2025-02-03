// tests/plugins_test.rs

use std::collections::HashMap;

use bodo::graph::{ConcurrentGroupData, Graph, Node, NodeKind, TaskData};
use bodo::plugin::Plugin;

#[test]
fn test_env_plugin() {
    let mut plugin = bodo::plugins::env_plugin::EnvPlugin::new();
    let config = bodo::plugin::PluginConfig {
        options: Some(
            serde_json::json!({
                "env": {
                    "GLOBAL_VAR": "global_value"
                }
            })
            .as_object()
            .cloned()
            .unwrap(),
        ),
        ..Default::default()
    };
    plugin.on_init(&config).unwrap();

    let mut graph = Graph::new();

    let task_data = TaskData {
        name: "test_task".to_string(),
        description: None,
        command: Some("echo $GLOBAL_VAR".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    };
    let node_id = graph.add_node(NodeKind::Task(task_data));
    plugin.on_graph_build(&mut graph).unwrap();

    let node = &graph.nodes[node_id as usize];
    if let NodeKind::Task(task_data) = &node.kind {
        assert_eq!(
            task_data.env.get("GLOBAL_VAR"),
            Some(&"global_value".to_string())
        );
    } else {
        panic!("Expected Task node");
    }
}

#[test]
fn test_execution_plugin() {
    use bodo::plugin::PluginConfig;
    use bodo::plugins::execution_plugin::ExecutionPlugin;

    // Build a graph with one task that echoes something
    let mut graph = Graph::new();

    let task_data = TaskData {
        name: "echo_task".to_string(),
        description: None,
        command: Some(
            "echo 'Hello from test_execution_plugin!' > /tmp/bodo_test_output".to_string(),
        ),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "".to_string(),
        watch: None,
    };

    let node_id = graph.add_node(NodeKind::Task(task_data));

    // Register task in task_registry
    graph.task_registry.insert("echo_task".to_string(), node_id);

    let mut plugin = ExecutionPlugin::new();

    let mut options = serde_json::Map::new();
    options.insert(
        "task".into(),
        serde_json::Value::String("echo_task".to_string()),
    );
    let config = PluginConfig {
        options: Some(options),
        ..Default::default()
    };

    plugin.on_init(&config).unwrap();

    // Run on_after_run to execute the task
    plugin.on_after_run(&mut graph).unwrap();

    // Verify that the command executed by checking the output file
    let output = std::fs::read_to_string("/tmp/bodo_test_output").unwrap();
    assert_eq!(output.trim(), "Hello from test_execution_plugin!");
    // Clean up
    let _ = std::fs::remove_file("/tmp/bodo_test_output");
}

#[test]
fn test_execution_plugin_with_concurrent_group() {
    use bodo::plugin::PluginConfig;
    use bodo::plugins::execution_plugin::ExecutionPlugin;
    use tempfile::TempDir;

    // Create a temporary directory
    let temp_dir = TempDir::new().unwrap();
    let temp_dir_path = temp_dir.path();

    let output_file1 = temp_dir_path.join("bodo_test_output_child1");
    let output_file2 = temp_dir_path.join("bodo_test_output_child2");

    // Adjust commands to write files using absolute paths
    let command1 = format!("echo Hello from child 1 > {}", output_file1.display());
    let command2 = format!("echo Hello from child 2 > {}", output_file2.display());

    // Build a graph with a concurrent group
    let mut graph = Graph::new();

    // Create tasks
    let task_data_main = TaskData {
        name: "main_task".to_string(),
        description: None,
        command: None, // No command
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        is_default: true,
        script_id: "script".to_string(),
        script_display_name: "".to_string(),
        watch: None,
    };

    let main_task_id = graph.add_node(NodeKind::Task(task_data_main));
    graph
        .task_registry
        .insert("main_task".to_string(), main_task_id);

    let task_data_child1 = TaskData {
        name: "child_task1".to_string(),
        description: None,
        command: Some(command1.clone()),
        working_dir: Some(temp_dir_path.to_str().unwrap().to_string()),
        env: HashMap::new(),
        exec_paths: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "".to_string(),
        watch: None,
    };

    let child1_id = graph.add_node(NodeKind::Task(task_data_child1));
    graph
        .task_registry
        .insert("child_task1".to_string(), child1_id);

    let task_data_child2 = TaskData {
        name: "child_task2".to_string(),
        description: None,
        command: Some(command2.clone()),
        working_dir: Some(temp_dir_path.to_str().unwrap().to_string()),
        env: HashMap::new(),
        exec_paths: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "".to_string(),
        watch: None,
    };

    let child2_id = graph.add_node(NodeKind::Task(task_data_child2));
    graph
        .task_registry
        .insert("child_task2".to_string(), child2_id);

    // Create a concurrent group node
    let group_data = ConcurrentGroupData {
        child_nodes: vec![child1_id, child2_id],
        fail_fast: true,
        max_concurrent: Some(2),
        timeout_secs: None,
    };
    let group_node = Node {
        id: graph.nodes.len() as u64,
        kind: NodeKind::ConcurrentGroup(group_data),
        metadata: HashMap::new(),
    };
    let group_id = group_node.id;
    graph.nodes.push(group_node);

    // Add edges
    graph.add_edge(main_task_id, group_id).unwrap();
    graph.add_edge(group_id, child1_id).unwrap();
    graph.add_edge(group_id, child2_id).unwrap();

    let mut plugin = ExecutionPlugin::new();
    let mut options = serde_json::Map::new();
    options.insert(
        "task".into(),
        serde_json::Value::String("main_task".to_string()),
    );
    let config = PluginConfig {
        options: Some(options),
        ..Default::default()
    };
    plugin.on_init(&config).unwrap();

    // Run on_after_run to execute the task
    plugin.on_after_run(&mut graph).unwrap();

    // Verify that the commands executed by checking the output files
    let output1 = std::fs::read_to_string(output_file1).expect("Failed to read output file 1");
    assert_eq!(output1.trim(), "Hello from child 1");

    let output2 = std::fs::read_to_string(output_file2).expect("Failed to read output file 2");
    assert_eq!(output2.trim(), "Hello from child 2");
    // Clean up automatically when temp_dir goes out of scope
}

// ... rest of the tests remain unchanged ...
