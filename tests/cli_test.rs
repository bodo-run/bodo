use bodo::cli::{get_task_name, Args};
use bodo::errors::BodoError;
use bodo::graph::{Node, NodeKind, TaskData};
use bodo::manager::GraphManager;
use clap::Parser;
use std::collections::HashMap;

#[test]
fn test_cli_parser() {
    let args = Args::parse_from([
        "bodo", "--debug", "-l", "mytask", "subtask", "--", "arg1", "arg2",
    ]);
    assert_eq!(args.task, Some("mytask".to_string()));
    assert_eq!(args.subtask, Some("subtask".to_string()));
    assert_eq!(args.args, vec!["arg1".to_string(), "arg2".to_string()]);
    assert!(args.debug);
    assert!(args.list);

    // Test default no-argument invocation.
    let default_args = Args::parse_from(["bodo"]);
    assert_eq!(default_args.task, None);
    assert_eq!(default_args.subtask, None);
    assert!(default_args.args.is_empty());
}

#[test]
fn test_get_task_name_default_task() -> Result<(), BodoError> {
    let mut manager = GraphManager::new();
    // Manually add a default task to the graph and registry:
    let default_task = TaskData {
        name: "default".to_string(),
        description: Some("Default Task".to_string()),
        command: Some("echo default".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: true,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
    };
    // Create a node with id 0.
    manager.graph.nodes.push(Node {
        id: 0,
        kind: NodeKind::Task(default_task),
        metadata: HashMap::new(),
    });
    manager.graph.task_registry.insert("default".to_string(), 0);

    // With no explicit task in CLI args:
    let args = Args {
        list: false,
        watch: false,
        auto_watch: false,
        debug: false,
        task: None,
        subtask: None,
        args: vec![],
    };
    let name = get_task_name(&args, &manager)?;
    assert_eq!(name, "default");
    Ok(())
}

#[test]
fn test_get_task_name_with_existing_task() -> Result<(), BodoError> {
    let mut manager = GraphManager::new();
    // Add task "build"
    let build_task = TaskData {
        name: "build".to_string(),
        description: Some("Build Task".to_string()),
        command: Some("cargo build".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
    };
    manager.graph.nodes.push(Node {
        id: 0,
        kind: NodeKind::Task(build_task),
        metadata: HashMap::new(),
    });
    manager.graph.task_registry.insert("build".to_string(), 0);
    let args = Args {
        list: false,
        watch: false,
        auto_watch: false,
        debug: false,
        task: Some("build".to_string()),
        subtask: None,
        args: vec![],
    };
    let name = get_task_name(&args, &manager)?;
    assert_eq!(name, "build");
    Ok(())
}
