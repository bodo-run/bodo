use bodo::config::{BodoConfig, TaskArgument, TaskConfig};
use bodo::errors::BodoError;
use bodo::graph::{CommandData, Graph, NodeKind, TaskData};
use bodo::script_loader::ScriptLoader;
use std::fs;
use std::path::Path;
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
    assert!(graph.task_registry.contains_key("test_task"));
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
    assert!(graph.task_registry.contains_key("task1"));
    assert!(graph.task_registry.contains_key("task2"));
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
    assert!(graph.task_registry.contains_key("task1"));
    assert!(graph.task_registry.contains_key("task2"));

    // Check that there's an edge from task2 to task1
    let task1_id = graph.task_registry.get("task1").unwrap();
    let task2_id = graph.task_registry.get("task2").unwrap();

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
fn test_parse_cross_file_ref() {
    let loader = ScriptLoader::new();
    let referencing_file = Path::new("dir/script.yaml");
    let dep = "../other.yaml/some-task";
    let result = loader.parse_cross_file_ref(dep, referencing_file);
    assert!(result.is_some());
    let (script_path, task_name) = result.unwrap();
    assert_eq!(script_path, Path::new("dir/../other.yaml"));
    assert_eq!(task_name, "some-task".to_string());
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
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: None,
    }));
    graph.add_edge(node_id1, node_id2).unwrap();
    graph.add_edge(node_id2, node_id1).unwrap();

    let cycle = graph.detect_cycle().unwrap();
    let error_msg = graph.format_cycle_error(&cycle);
    assert!(
        error_msg.contains("task1") && error_msg.contains("task2"),
        "Error message should include task1 and task2"
    );
}

#[test]
fn test_add_invalid_edge() {
    let mut graph = Graph::new();
    let result = graph.add_edge(10, 20);
    assert!(result.is_err());
}

#[test]
fn test_topological_sort() {
    let mut graph = Graph::new();
    let node_a = graph.add_node(NodeKind::Task(TaskData {
        name: "A".to_string(),
        description: None,
        command: None,
        working_dir: None,
        env: Default::default(),
        exec_paths: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
    }));
    let node_b = graph.add_node(NodeKind::Task(TaskData {
        name: "B".to_string(),
        description: None,
        command: None,
        working_dir: None,
        env: Default::default(),
        exec_paths: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
    }));
    let node_c = graph.add_node(NodeKind::Task(TaskData {
        name: "C".to_string(),
        description: None,
        command: None,
        working_dir: None,
        env: Default::default(),
        exec_paths: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
    }));
    graph.add_edge(node_a, node_b).unwrap();
    graph.add_edge(node_b, node_c).unwrap();

    let sorted = graph.topological_sort().unwrap();
    assert_eq!(sorted.len(), 3);
    assert!(sorted[0] == node_a && sorted[1] == node_b && sorted[2] == node_c);
}

impl PartialEq for TaskData {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.description == other.description
            && self.command == other.command
            && self.working_dir == other.working_dir
            && self.env == other.env
            && self.exec_paths == other.exec_paths
            && self.is_default == other.is_default
            && self.script_id == other.script_id
            && self.script_display_name == other.script_display_name
            && self.watch == other.watch
    }
}

#[test]
fn test_invalid_task_config() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    // Invalid task (no command or dependencies)
    let script_content = r#"
    tasks:
      invalid_task:
        description: "This task has no command or dependencies."
    "#;

    fs::write(&script_path, script_content).unwrap();

    let mut loader = ScriptLoader::new();
    let mut config = BodoConfig::default();
    config.scripts_dirs = Some(vec![temp_dir.path().to_string_lossy().to_string()]);

    let result = loader.build_graph(config);
    assert!(
        result.is_err(),
        "Invalid task configuration was not detected"
    );
    match result {
        Err(BodoError::ValidationError(msg)) => {
            assert!(
                msg.contains("A task must have a command or some dependencies"),
                "Incorrect validation error message"
            );
        }
        _ => panic!("Expected ValidationError"),
    }
}

#[test]
fn test_parse_cross_file_ref_no_slash() {
    let loader = ScriptLoader::new();
    let referencing_file = Path::new("dir/script.yaml");
    let dep = "task-name";
    let result = loader.parse_cross_file_ref(dep, referencing_file);
    assert!(result.is_none());
}

#[test]
fn test_generate_schema() {
    let schema = BodoConfig::generate_schema();
    assert!(!schema.is_empty(), "Schema should not be empty");
    // Optionally, verify that the schema contains certain expected strings
    assert!(
        schema.contains("\"title\": \"BodoConfig\""),
        "Schema should contain BodoConfig title"
    );
}

#[test]
fn test_parse_cross_file_ref_invalid() {
    let loader = ScriptLoader::new();
    let referencing_file = Path::new("dir/script.yaml");
    let dep = "invalid-dep-format";
    let result = loader.parse_cross_file_ref(dep, referencing_file);
    assert!(result.is_none());
}

#[test]
fn test_parse_cross_file_ref_valid() {
    let loader = ScriptLoader::new();
    let referencing_file = Path::new("dir/script.yaml");
    let dep = "../other.yaml/some-task";
    let result = loader.parse_cross_file_ref(dep, referencing_file);
    assert!(result.is_some());
    let (script_path, task_name) = result.unwrap();
    assert_eq!(script_path, Path::new("dir/../other.yaml"));
    assert_eq!(task_name, "some-task".to_string());
}

#[test]
fn test_parse_cross_file_ref_multiple_slashes() {
    let loader = ScriptLoader::new();
    let referencing_file = Path::new("dir/subdir/script.yaml");
    let dep = "../../other.yaml/some/task";
    let result = loader.parse_cross_file_ref(dep, referencing_file);
    assert!(result.is_some());
    let (script_path, task_name) = result.unwrap();
    assert_eq!(script_path, Path::new("dir/subdir/../../other.yaml"));
    assert_eq!(task_name, "some/task".to_string());
}

#[test]
fn test_parse_cross_file_ref_trailing_slash() {
    let loader = ScriptLoader::new();
    let referencing_file = Path::new("dir/script.yaml");
    let dep = "../other.yaml/some-task/";
    let result = loader.parse_cross_file_ref(dep, referencing_file);
    assert!(result.is_some());
    let (script_path, task_name) = result.unwrap();
    assert_eq!(script_path, Path::new("dir/../other.yaml"));
    assert_eq!(task_name, "some-task/".to_string());
}
