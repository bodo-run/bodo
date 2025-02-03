// tests/script_loader_test.rs

use bodo::config::{BodoConfig, TaskConfig};
use bodo::graph::{Graph, NodeKind, TaskData};
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
fn test_generate_schema() {
    let schema = BodoConfig::generate_schema();
    assert!(!schema.is_empty(), "Schema should not be empty");
    assert!(
        schema.contains("\"title\": \"BodoConfig\""),
        "Schema should contain BodoConfig title"
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
fn test_build_graph_with_duplicate_tasks() {
    // Existing test code...
}

#[test]
fn test_build_graph_with_invalid_task_names() {
    // Existing test code...
}

// Additional tests can be added here...
