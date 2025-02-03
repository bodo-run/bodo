use std::collections::HashMap;

use bodo::graph::{ConcurrentGroupData, Graph, Node, NodeKind, TaskData};
use bodo::plugin::Plugin;

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
        command: Some(format!(
            "touch {}",
            output_file1.file_name().unwrap().to_str().unwrap()
        )),
        working_dir: Some(temp_dir_path.to_string_lossy().to_string()),
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
        command: Some(format!(
            "touch {}",
            output_file2.file_name().unwrap().to_str().unwrap()
        )),
        working_dir: Some(temp_dir_path.to_string_lossy().to_string()),
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
    assert!(output_file1.exists(), "Output file 1 was not created");
    assert!(output_file2.exists(), "Output file 2 was not created");
}
