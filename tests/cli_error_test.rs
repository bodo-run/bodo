use bodo::cli::{get_task_name, Args};
use bodo::errors::BodoError;
use bodo::manager::GraphManager;

#[test]
fn test_get_task_name_no_default_and_no_argument() {
    // GraphManager has no default task
    let manager = GraphManager::new();
    let args = Args {
        list: false,
        watch: false,
        auto_watch: false,
        debug: false,
        task: None,
        subtask: None,
        args: vec![],
        dry_run: false,
    };
    let res = get_task_name(&args, &manager);
    match res {
        Err(BodoError::NoTaskSpecified) => {}
        _ => panic!("Expected NoTaskSpecified error when no default task is present"),
    }
}

#[test]
fn test_get_task_name_task_not_found() {
    // GraphManager contains a task "dummy" only.
    let mut manager = GraphManager::new();
    let dummy_id = manager
        .graph
        .add_node(bodo::graph::NodeKind::Task(bodo::graph::TaskData {
            name: "dummy".to_string(),
            description: None,
            command: Some("echo dummy".to_string()),
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
        }));
    manager
        .graph
        .task_registry
        .insert("dummy".to_string(), dummy_id);

    let args = Args {
        list: false,
        watch: false,
        auto_watch: false,
        debug: false,
        task: Some("nonexistent".to_string()),
        subtask: None,
        args: vec![],
        dry_run: false,
    };
    let res = get_task_name(&args, &manager);
    match res {
        Err(BodoError::TaskNotFound(_)) => {}
        _ => panic!("Expected TaskNotFound error for a task that does not exist"),
    }
}
