use bodo::cli::{get_task_name, Args};
use bodo::manager::GraphManager;
use bodo::BodoError;

#[test]
fn test_get_task_name_default() {
    let args = Args {
        list: false,
        watch: false,
        auto_watch: false,
        debug: false,
        task: None,
        subtask: None,
        args: vec![],
    };
    let mut manager = GraphManager::new();
    manager.graph.task_registry.insert("default".to_string(), 0);
    assert_eq!(get_task_name(&args, &manager).unwrap(), "default");
}

#[test]
fn test_get_task_name_with_subtask() {
    let args = Args {
        task: Some("main".to_string()),
        subtask: Some("build".to_string()),
        list: false,
        watch: false,
        auto_watch: false,
        debug: false,
        args: vec![],
    };
    let mut manager = GraphManager::new();
    manager
        .graph
        .task_registry
        .insert("main build".to_string(), 0);
    assert_eq!(get_task_name(&args, &manager).unwrap(), "main build");
}

#[test]
fn test_get_task_name_not_found() {
    let args = Args {
        task: Some("missing".to_string()),
        list: false,
        watch: false,
        auto_watch: false,
        debug: false,
        subtask: None,
        args: vec![],
    };
    let manager = GraphManager::new();
    let result = get_task_name(&args, &manager);
    assert!(matches!(result, Err(BodoError::TaskNotFound(_))));
}
