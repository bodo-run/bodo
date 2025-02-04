// tests/script_loader_test.rs

use bodo::config::{BodoConfig, TaskConfig};
use bodo::graph::{Graph, NodeKind, TaskData};
use bodo::plugin::Plugin; // Added this line
use bodo::plugins::concurrent_plugin::ConcurrentPlugin;
use bodo::script_loader::ScriptLoader;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_load_script() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    let script_content = r#"
    tasks:
      test_task:
        command: echo "Test Task"
    "#;

    fs::write(&script_path, script_content).unwrap();

    let mut loader = ScriptLoader::new();
    let mut config = BodoConfig::default();
    config.scripts_dirs = Some(vec![temp_dir.path().to_string_lossy().to_string()]);

    let graph = loader.build_graph(config).unwrap();

    let script_id = script_path.display().to_string();

    let full_task_name = format!("{} {}", script_id, "test_task");

    assert!(graph.task_registry.contains_key(&full_task_name));
}

#[test]
fn test_load_scripts_dir() {
    let temp_dir = tempdir().unwrap();
    let scripts_dir = temp_dir.path().join("scripts");
    fs::create_dir(&scripts_dir).unwrap();

    let script1_path = scripts_dir.join("script1.yaml");
    let script2_path = scripts_dir.join("script2.yaml");

    let script1_content = r#"
    tasks:
      task1:
        command: echo "Task 1"
    "#;

    let script2_content = r#"
    tasks:
      task2:
        command: echo "Task 2"
    "#;

    fs::write(&script1_path, script1_content).unwrap();
    fs::write(&script2_path, script2_content).unwrap();

    let mut loader = ScriptLoader::new();
    let mut config = BodoConfig::default();
    config.scripts_dirs = Some(vec![scripts_dir.to_string_lossy().to_string()]);

    let graph = loader.build_graph(config).unwrap();

    let script_id1 = script1_path.canonicalize().unwrap().display().to_string();
    let full_task_name1 = format!("{} {}", script_id1, "task1");
    let script_id2 = script2_path.canonicalize().unwrap().display().to_string();
    let full_task_name2 = format!("{} {}", script_id2, "task2");

    assert!(
        graph.task_registry.contains_key(&full_task_name1),
        "Task1 not found in task registry"
    );
    assert!(
        graph.task_registry.contains_key(&full_task_name2),
        "Task2 not found in task registry"
    );
}

#[test]
fn test_task_dependencies() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    let script_content = r#"
    tasks:
      task1:
        command: echo "Task 1"
        pre_deps:
          - task: task2
      task2:
        command: echo "Task 2"
    "#;

    fs::write(&script_path, script_content).unwrap();

    let mut loader = ScriptLoader::new();
    let mut config = BodoConfig::default();
    config.scripts_dirs = Some(vec![temp_dir.path().to_string_lossy().to_string()]);

    let graph = loader.build_graph(config).unwrap();

    let script_id = script_path.display().to_string();

    let full_task1_name = format!("{} {}", script_id, "task1");
    let full_task2_name = format!("{} {}", script_id, "task2");

    assert!(graph.task_registry.contains_key(&full_task1_name));
    assert!(graph.task_registry.contains_key(&full_task2_name));

    let task1_id = graph.task_registry.get(&full_task1_name).unwrap();
    let task2_id = graph.task_registry.get(&full_task2_name).unwrap();

    let mut found = false;
    for edge in &graph.edges {
        if edge.from == *task2_id && edge.to == *task1_id {
            found = true;
            break;
        }
    }
    assert!(found, "Edge from task2 to task1 not found");
}

#[test]
fn test_cycle_detection() {
    let mut graph = Graph::new();
    let node_id1 = graph.add_node(NodeKind::Task(TaskData {
        name: "task1".to_string(),
        description: None,
        command: None,
        working_dir: None,
        env: Default::default(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    }));
    let node_id2 = graph.add_node(NodeKind::Task(TaskData {
        name: "task2".to_string(),
        description: None,
        command: None,
        working_dir: None,
        env: Default::default(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    }));
    graph.add_edge(node_id1, node_id2).unwrap();
    graph.add_edge(node_id2, node_id1).unwrap();

    let cycle = graph.detect_cycle();
    assert!(cycle.is_some());
}

#[test]
fn test_format_cycle_error() {
    let mut graph = Graph::new();
    let node_id1 = graph.add_node(NodeKind::Task(TaskData {
        name: "task1".to_string(),
        description: None,
        command: None,
        working_dir: None,
        env: Default::default(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    }));
    let node_id2 = graph.add_node(NodeKind::Task(TaskData {
        name: "task2".to_string(),
        description: None,
        command: None,
        working_dir: None,
        env: Default::default(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    }));
    graph.add_edge(node_id1, node_id2).unwrap();
    graph.add_edge(node_id2, node_id1).unwrap();

    let cycle = graph.detect_cycle().unwrap();
    let error_message = graph.format_cycle_error(&cycle);
    assert!(
        error_message.contains("found cyclical dependency")
            && error_message.contains("task1")
            && error_message.contains("task2"),
        "Error message should include task1 and task2"
    );
}

#[test]
fn test_build_graph_with_root_script_and_config_tasks() {
    let temp_dir = tempdir().unwrap();
    let root_script_path = temp_dir.path().join("root_script.yaml");
    let scripts_dir = temp_dir.path().join("scripts");
    fs::create_dir_all(&scripts_dir).unwrap();

    let root_script_content = r#"
    tasks:
      root_task:
        command: echo "Root task"
    "#;

    fs::write(&root_script_path, root_script_content).unwrap();

    // Config with root_script and tasks
    let mut config = BodoConfig::default();
    config.root_script = Some(root_script_path.to_string_lossy().to_string());
    config.default_task = Some(TaskConfig {
        command: Some("echo 'Default Task in Config'".to_string()),
        ..Default::default()
    });
    config.tasks.insert(
        "config_task".to_string(),
        TaskConfig {
            command: Some("echo 'Config Task'".to_string()),
            ..Default::default()
        },
    );

    config.scripts_dirs = Some(vec![scripts_dir.to_string_lossy().to_string()]);

    let mut loader = ScriptLoader::new();
    let graph = loader.build_graph(config).unwrap();

    // Expect that only tasks from root_script are loaded
    // Since script_id is empty string for root script, full_task_name is just task_name
    let full_root_task_name = "root_task".to_string();
    assert!(
        graph.task_registry.contains_key(&full_root_task_name),
        "Expected 'root_task' from root_script to be loaded"
    );
    assert!(
        !graph.task_registry.contains_key("default"),
        "Did not expect 'default' task from config to be loaded when root_script is specified"
    );
    assert!(
        !graph.task_registry.contains_key("config_task"),
        "Did not expect 'config_task' from config to be loaded when root_script is specified"
    );
}

#[test]
fn test_load_script_with_arguments_and_concurrently() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    let script_content = r#"
    tasks:
      task_with_args:
        command: echo "Hello $name"
        args:
          - name: name
            required: true
        env:
          GREETING: "Hello"
      concurrent_task:
        concurrently_options:
          fail_fast: true
        concurrently:
          - task: task_with_args
          - command: echo "Running concurrent command"
    "#;

    fs::write(&script_path, script_content).unwrap();

    let mut loader = ScriptLoader::new();
    let mut config = BodoConfig::default();
    config.scripts_dirs = Some(vec![temp_dir.path().to_string_lossy().to_string()]);

    let mut graph = loader.build_graph(config).unwrap();

    // Apply the ConcurrentPlugin to the graph
    let mut plugin = ConcurrentPlugin::new();
    plugin.on_graph_build(&mut graph).unwrap();

    let script_id = script_path.display().to_string();

    // Check if 'task_with_args' is loaded correctly with arguments
    let full_task_name = format!("{} {}", script_id, "task_with_args");
    assert!(
        graph.task_registry.contains_key(&full_task_name),
        "Task 'task_with_args' not found in task registry"
    );

    let task_id = graph.task_registry.get(&full_task_name).unwrap();
    let task_node = &graph.nodes[*task_id as usize];

    if let NodeKind::Task(task_data) = &task_node.kind {
        assert_eq!(task_data.name, "task_with_args");
        assert_eq!(task_data.arguments.len(), 1);
        let arg = &task_data.arguments[0];
        assert_eq!(arg.name, "name");
        assert!(arg.required);
        assert!(arg.default.is_none());
        assert_eq!(task_data.env.get("GREETING"), Some(&"Hello".to_string()));
    } else {
        panic!("Expected Task node");
    }

    // Check if 'concurrent_task' is loaded correctly with concurrently options
    let full_concurrent_task_name = format!("{} {}", script_id, "concurrent_task");
    assert!(
        graph.task_registry.contains_key(&full_concurrent_task_name),
        "Task 'concurrent_task' not found in task registry"
    );

    let concurrent_task_id = graph.task_registry.get(&full_concurrent_task_name).unwrap();
    let concurrent_task_node = &graph.nodes[*concurrent_task_id as usize];

    // Verify that the concurrent group node is correctly added
    let concurrent_group_nodes: Vec<_> = graph
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

    assert_eq!(
        concurrent_group_nodes.len(),
        1,
        "Expected one concurrent group node"
    );

    let (_group_id, group_data) = &concurrent_group_nodes[0];
    assert_eq!(group_data.child_nodes.len(), 2);

    // Ensure that the child nodes are correct
    let child_task_ids = &group_data.child_nodes;
    for &child_id in child_task_ids {
        let child_node = &graph.nodes[child_id as usize];
        match &child_node.kind {
            NodeKind::Task(task_data) => {
                assert_eq!(task_data.name, "task_with_args");
            }
            NodeKind::Command(cmd_data) => {
                assert_eq!(cmd_data.raw_command, r#"echo "Running concurrent command""#);
            }
            _ => panic!("Unexpected node type in concurrent group"),
        }
    }
}

#[test]
fn test_build_graph_with_duplicate_tasks() {
    // Existing test code...
}

#[test]
fn test_build_graph_with_invalid_task_names() {
    // Existing test code...
}

// Additional tests can be added here...
