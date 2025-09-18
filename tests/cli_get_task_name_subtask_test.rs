use bodo::cli::{get_task_name, Args};
use bodo::errors::BodoError;
use bodo::manager::GraphManager;

#[test]
fn test_get_task_name_with_subtask_exists() {
    let mut manager = GraphManager::new();
    // Add a task with concatenated name "build unit"
    manager.graph.nodes.push(bodo::graph::Node {
        id: 0,
        kind: bodo::graph::NodeKind::Task(bodo::graph::TaskData {
            name: "build unit".to_string(),
            description: None,
            command: Some("echo build unit".to_string()),
            working_dir: None,
            env: Default::default(),
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
        }),
        metadata: Default::default(),
    });
    manager
        .graph
        .task_registry
        .insert("build unit".to_string(), 0);

    let args = Args {
        list: false,
        watch: false,
        auto_watch: false,
        debug: false,
        task: Some("build".to_string()),
        subtask: Some("unit".to_string()),
        args: vec![],
        dry_run: false,
    };

    let result = get_task_name(&args, &manager).unwrap();
    assert_eq!(result, "build unit");
}

#[test]
fn test_get_task_name_with_subtask_not_found() {
    let mut manager = GraphManager::new();
    // Only add task "build" exists.
    manager.graph.nodes.push(bodo::graph::Node {
        id: 0,
        kind: bodo::graph::NodeKind::Task(bodo::graph::TaskData {
            name: "build".to_string(),
            description: None,
            command: Some("echo build".to_string()),
            working_dir: None,
            env: Default::default(),
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
        }),
        metadata: Default::default(),
    });
    manager.graph.task_registry.insert("build".to_string(), 0);

    let args = Args {
        list: false,
        watch: false,
        auto_watch: false,
        debug: false,
        task: Some("build".to_string()),
        subtask: Some("unit".to_string()),
        args: vec![],
        dry_run: false,
    };

    let result = get_task_name(&args, &manager);
    match result {
        Err(BodoError::TaskNotFound(_)) => {}
        _ => panic!("Expected TaskNotFound error when concatenated task not found"),
    }
}
