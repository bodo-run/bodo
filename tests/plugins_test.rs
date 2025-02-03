// tests/plugins_test.rs

use bodo::graph::{ConcurrentGroupData, Graph, Node, NodeKind, TaskData};
use bodo::plugin::{Plugin, PluginConfig};
use bodo::plugins::concurrent_plugin::ConcurrentPlugin;
use bodo::plugins::env_plugin::EnvPlugin;
use bodo::plugins::execution_plugin::expand_env_vars;
use bodo::plugins::path_plugin::PathPlugin;
use bodo::plugins::prefix_plugin::PrefixPlugin;
use bodo::plugins::print_list_plugin::PrintListPlugin;
use bodo::plugins::timeout_plugin::TimeoutPlugin;
use bodo::plugins::watch_plugin::WatchPlugin;
use bodo::BodoError;
use std::collections::HashMap;
use tempfile::TempDir;

#[test]
fn test_concurrent_plugin() {
    let mut plugin = ConcurrentPlugin::new();

    let mut graph = Graph::new();

    // Create tasks
    let task_data_main = TaskData {
        name: "main_task".to_string(),
        description: None,
        command: None, // No command, will have concurrent tasks
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        is_default: true,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    };

    let main_task_id = graph.add_node(NodeKind::Task(task_data_main));

    let task_data_child1 = TaskData {
        name: "child_task1".to_string(),
        description: None,
        command: Some("echo Child 1".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    };

    let child1_id = graph.add_node(NodeKind::Task(task_data_child1));

    let task_data_child2 = TaskData {
        name: "child_task2".to_string(),
        description: None,
        command: Some("echo Child 2".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    };

    let child2_id = graph.add_node(NodeKind::Task(task_data_child2));

    // Register child tasks in task_registry
    graph
        .task_registry
        .insert("child_task1".to_string(), child1_id);
    graph
        .task_registry
        .insert("child_task2".to_string(), child2_id);

    // Set up the main_task to have concurrent tasks
    let main_node = &mut graph.nodes[main_task_id as usize];
    // Set the metadata 'concurrently' directly as a JSON array string
    main_node.metadata.insert(
        "concurrently".to_string(),
        serde_json::to_string(&["child_task1", "child_task2"]).unwrap_or_default(),
    );

    // Apply the plugin
    let result = plugin.on_graph_build(&mut graph);
    assert!(
        result.is_ok(),
        "Plugin on_graph_build returned an error: {:?}",
        result.unwrap_err()
    );

    // Check that a ConcurrentGroup node has been added
    let group_nodes: Vec<_> = graph
        .nodes
        .iter()
        .filter_map(|node| {
            if let NodeKind::ConcurrentGroup(group_data) = &node.kind {
                Some((node.id, group_data))
            } else {
                None
            }
        })
        .collect();

    assert_eq!(group_nodes.len(), 1, "Expected one concurrent group node");

    let (group_id, group_data) = &group_nodes[0];
    assert_eq!(group_data.child_nodes.len(), 2);
    assert!(group_data.child_nodes.contains(&child1_id));
    assert!(group_data.child_nodes.contains(&child2_id));

    // Check that edges have been added appropriately
    // Edge from main_task to group
    assert!(graph
        .edges
        .iter()
        .any(|edge| edge.from == main_task_id && edge.to == *group_id));

    // Edges from group to child tasks
    assert!(graph
        .edges
        .iter()
        .any(|edge| edge.from == *group_id && edge.to == child1_id));
    assert!(graph
        .edges
        .iter()
        .any(|edge| edge.from == *group_id && edge.to == child2_id));
}

#[test]
fn test_prefix_plugin_on_graph_build() {
    let mut plugin = PrefixPlugin::new();
    let mut graph = Graph::new();

    // Create a ConcurrentGroup node with prefix_output metadata
    let group_node_id = graph.add_node(NodeKind::ConcurrentGroup(ConcurrentGroupData {
        child_nodes: vec![],
        fail_fast: true,
        max_concurrent: None,
        timeout_secs: None,
    }));
    graph.nodes[group_node_id as usize]
        .metadata
        .insert("prefix_output".to_string(), "true".to_string());

    // Add child tasks
    let task1_id = graph.add_node(NodeKind::Task(TaskData {
        name: "task1".to_string(),
        description: None,
        command: Some("echo Task 1".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    }));

    let task2_id = graph.add_node(NodeKind::Task(TaskData {
        name: "task2".to_string(),
        description: None,
        command: Some("echo Task 2".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    }));

    // Add child nodes to the group
    if let NodeKind::ConcurrentGroup(group_data) = &mut graph.nodes[group_node_id as usize].kind {
        group_data.child_nodes.push(task1_id);
        group_data.child_nodes.push(task2_id);
    }

    // Run the plugin
    let result = plugin.on_graph_build(&mut graph);
    assert!(result.is_ok());

    // Check that child tasks have prefix metadata
    let child1 = &graph.nodes[task1_id as usize];
    assert_eq!(
        child1.metadata.get("prefix_enabled"),
        Some(&"true".to_string())
    );
    assert!(child1.metadata.get("prefix_label").is_some());
    assert!(child1.metadata.get("prefix_color").is_some());

    let child2 = &graph.nodes[task2_id as usize];
    assert_eq!(
        child2.metadata.get("prefix_enabled"),
        Some(&"true".to_string())
    );
    assert!(child2.metadata.get("prefix_label").is_some());
    assert!(child2.metadata.get("prefix_color").is_some());
}

#[test]
fn test_print_list_plugin() {
    let mut plugin = PrintListPlugin;

    let mut graph = Graph::new();

    // Create two tasks with different properties
    let task_data1 = TaskData {
        name: "task1".to_string(),
        description: Some("First task".to_string()),
        command: Some("echo Task 1".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        is_default: false,
        script_id: "script1".to_string(),
        script_display_name: "Script 1".to_string(),
        watch: None,
    };

    let task_data2 = TaskData {
        name: "task2".to_string(),
        description: Some("Second task".to_string()),
        command: Some("echo Task 2".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        is_default: false,
        script_id: "script2".to_string(),
        script_display_name: "Script 2".to_string(),
        watch: None,
    };

    // Add nodes to graph
    graph.add_node(NodeKind::Task(task_data1));
    graph.add_node(NodeKind::Task(task_data2));

    // Apply the plugin
    let result = plugin.on_graph_build(&mut graph);
    assert!(result.is_ok(), "Plugin execution failed");
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
        options: Some(options),
        ..Default::default()
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
fn test_execution_plugin_with_command_node() -> Result<()> {
    let mut plugin = ExecutionPlugin::new();
    plugin.task_name = Some("test_task".to_string());
    let mut graph = Graph::new();
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "test_task".to_string(),
        description: None,
        command: None,
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        is_default: true,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
    }));
    graph.task_registry.insert("test_task".to_string(), task_id);
    let command_id = graph.add_node(NodeKind::Command(bodo::graph::CommandData {
        raw_command: "echo 'Command Node'".to_string(),
        description: None,
        working_dir: None,
        env: HashMap::new(),
        watch: None,
    }));
    graph.add_edge(task_id, command_id)?;
    let result = plugin.on_after_run(&mut graph);
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_expand_env_vars_basic() {
    let env_map = HashMap::from([
        ("VAR1".to_string(), "value1".to_string()),
        ("VAR2".to_string(), "value2".to_string()),
    ]);
    let input = "echo $VAR1 and $VAR2";
    let expected = "echo value1 and value2";
    let result = expand_env_vars(input, &env_map);
    assert_eq!(result, expected);
}

#[test]
fn test_expand_env_vars_no_match() {
    let env_map = HashMap::from([("VAR1".to_string(), "value1".to_string())]);
    let input = "echo $VAR2 and ${VAR3}";
    let expected = "echo $VAR2 and ${VAR3}";
    let result = expand_env_vars(input, &env_map);
    assert_eq!(result, expected);
}

#[test]
fn test_expand_env_vars_partial() {
    let env_map = HashMap::from([("HOME".to_string(), "/home/user".to_string())]);
    let input = "cd $HOME/projects";
    let expected = "cd /home/user/projects";
    let result = expand_env_vars(input, &env_map);
    assert_eq!(result, expected);
}

#[test]
fn test_expand_env_vars_special_chars() {
    let env_map = HashMap::from([("VAR".to_string(), "value".to_string())]);
    let input = "echo $$VAR $VAR$ $VAR text";
    let expected = "echo $VAR value$ value text";
    let result = expand_env_vars(input, &env_map);
    assert_eq!(result, expected);
}

#[test]
fn test_expand_env_vars_empty_var() {
    let env_map = HashMap::new();
    let input = "echo $";
    let expected = "echo $";
    let result = expand_env_vars(input, &env_map);
    assert_eq!(result, expected);
}

#[test]
fn test_execution_plugin_with_concurrent_group() -> Result<()> {
    use bodo::plugin::PluginConfig;
    use bodo::plugins::execution_plugin::ExecutionPlugin;
    use tempfile::TempDir;

    // Create a temporary directory
    let temp_dir = TempDir::new().unwrap();
    let temp_dir_path = temp_dir.path();

    // Define output file paths within the temporary directory
    let output_file1_name = "bodo_test_output_child1";
    let output_file2_name = "bodo_test_output_child2";
    let output_file1 = temp_dir_path.join(output_file1_name);
    let output_file2 = temp_dir_path.join(output_file2_name);

    // Adjust commands to write files using absolute paths
    let command1 = format!(
        "echo 'Hello from child 1' > \"{}\"",
        output_file1.to_string_lossy()
    );
    let command2 = format!(
        "echo 'Hello from child 2' > \"{}\"",
        output_file2.to_str().unwrap()
    );

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
        working_dir: None, // Not setting working_dir
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
        working_dir: None, // Not setting working_dir
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
    let output1 = std::fs::read_to_string(&output_file1)
        .map_err(|e| BodoError::PluginError(format!("Failed to read output file 1: {}", e)))?;
    assert_eq!(output1.trim(), "Hello from child 1");

    let output2 = std::fs::read_to_string(&output_file2)
        .map_err(|e| BodoError::PluginError(format!("Failed to read output file 2: {}", e)))?;
    assert_eq!(output2.trim(), "Hello from child 2");
    // Clean up
    let _ = temp_dir.close();
    Ok(())
}

#[test]
fn test_expand_env_vars_basic() {
    let env_map = HashMap::from([
        ("VAR1".to_string(), "value1".to_string()),
        ("VAR2".to_string(), "value2".to_string()),
    ]);
    let input = "echo $VAR1 and $VAR2";
    let expected = "echo value1 and value2";
    let result = expand_env_vars(input, &env_map);
    assert_eq!(result, expected);
}

#[test]
fn test_expand_env_vars_no_match() {
    let env_map = HashMap::from([("VAR1".to_string(), "value1".to_string())]);
    let input = "echo $VAR2 and ${VAR3}";
    let expected = "echo $VAR2 and ${VAR3}";
    let result = expand_env_vars(input, &env_map);
    assert_eq!(result, expected);
}

#[test]
fn test_expand_env_vars_partial() {
    let env_map = HashMap::from([("HOME".to_string(), "/home/user".to_string())]);
    let input = "cd $HOME/projects";
    let expected = "cd /home/user/projects";
    let result = expand_env_vars(input, &env_map);
    assert_eq!(result, expected);
}

#[test]
fn test_expand_env_vars_special_chars() {
    let env_map = HashMap::from([("VAR".to_string(), "value".to_string())]);
    let input = "echo $$VAR $VAR$ $VAR text";
    let expected = "echo $VAR value$ value text";
    let result = expand_env_vars(input, &env_map);
    assert_eq!(result, expected);
}

#[test]
fn test_expand_env_vars_empty_var() {
    let env_map = HashMap::new();
    let input = "echo $";
    let expected = "echo $";
    let result = expand_env_vars(input, &env_map);
    assert_eq!(result, expected);
}
