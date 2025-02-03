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
        command: Some("touch /tmp/bodo_test_output_child1 && echo 'Hello from child 1' > /tmp/bodo_test_output_child1".to_string()),
        working_dir: None,
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
        command: Some("touch /tmp/bodo_test_output_child2 && echo 'Hello from child 2' > /tmp/bodo_test_output_child2".to_string()),
        working_dir: None,
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

    // Give the commands some time to complete
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Print debug information
    println!("Checking if output files exist:");
    println!(
        "File 1: {}",
        std::path::Path::new("/tmp/bodo_test_output_child1").exists()
    );
    println!(
        "File 2: {}",
        std::path::Path::new("/tmp/bodo_test_output_child2").exists()
    );

    // Verify that the commands executed by checking the output files
    let output1 = std::fs::read_to_string("/tmp/bodo_test_output_child1")
        .expect("Failed to read output file 1");
    assert_eq!(output1.trim(), "Hello from child 1");
    let output2 = std::fs::read_to_string("/tmp/bodo_test_output_child2")
        .expect("Failed to read output file 2");
    assert_eq!(output2.trim(), "Hello from child 2");
    // Clean up
    let _ = std::fs::remove_file("/tmp/bodo_test_output_child1");
    let _ = std::fs::remove_file("/tmp/bodo_test_output_child2");
}

#[test]
fn test_watch_plugin() {
    use bodo::plugin::PluginConfig;
    use bodo::plugins::watch_plugin::WatchPlugin;

    std::env::set_var("BODO_NO_WATCH", "1");
    let mut plugin = WatchPlugin::new(false, false); // Start with watch_mode = false
    let config = PluginConfig {
        watch: false, // Don't enable watch mode
        ..Default::default()
    };
    plugin.on_init(&config).unwrap();

    let mut graph = Graph::new();

    let task_data = TaskData {
        name: "watch_task".to_string(),
        description: None,
        command: Some("echo 'Watching files'".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "".to_string(),
        watch: Some(bodo::config::WatchConfig {
            patterns: vec!["src/**/*.rs".to_string()],
            debounce_ms: 500,
            ignore_patterns: vec![],
            auto_watch: true,
        }),
    };

    let _node_id = graph.add_node(NodeKind::Task(task_data));
    plugin.on_graph_build(&mut graph).unwrap();

    // Test that the watch entries were set up correctly
    assert_eq!(plugin.get_watch_entry_count(), 0);
}

#[test]
fn test_prefix_plugin() {
    use bodo::graph::{ConcurrentGroupData, Node, NodeKind};
    use bodo::plugins::prefix_plugin::PrefixPlugin;

    let mut plugin = PrefixPlugin::new();

    let mut graph = Graph::new();

    let child_task1 = TaskData {
        name: "task1".to_string(),
        description: None,
        command: Some("echo Task 1".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        is_default: false,
        script_id: "script1".to_string(),
        script_display_name: "Script 1".to_string(),
        watch: None,
    };
    let child_id1 = graph.add_node(NodeKind::Task(child_task1));
    graph.task_registry.insert("task1".to_string(), child_id1);

    let child_task2 = TaskData {
        name: "task2".to_string(),
        description: None,
        command: Some("echo Task 2".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        is_default: false,
        script_id: "script1".to_string(),
        script_display_name: "Script 1".to_string(),
        watch: None,
    };
    let child_id2 = graph.add_node(NodeKind::Task(child_task2));
    graph.task_registry.insert("task2".to_string(), child_id2);

    let group_data = ConcurrentGroupData {
        child_nodes: vec![child_id1, child_id2],
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

    // Set prefix_output metadata on the group node
    graph.nodes[group_id as usize]
        .metadata
        .insert("prefix_output".to_string(), "true".to_string());

    plugin.on_graph_build(&mut graph).unwrap();

    // Check that child nodes have prefix metadata
    let node1 = &graph.nodes[child_id1 as usize];
    assert_eq!(
        node1.metadata.get("prefix_enabled"),
        Some(&"true".to_string())
    );
    assert!(node1.metadata.contains_key("prefix_label"));
    assert!(node1.metadata.contains_key("prefix_color"));

    let node2 = &graph.nodes[child_id2 as usize];
    assert_eq!(
        node2.metadata.get("prefix_enabled"),
        Some(&"true".to_string())
    );
    assert!(node2.metadata.contains_key("prefix_label"));
    assert!(node2.metadata.contains_key("prefix_color"));
}
