use bodo::cli::{get_task_name, Args};
use bodo::errors::BodoError;
use bodo::manager::GraphManager;

#[test]
fn test_get_task_name_with_task_and_subtask() {
    let args = Args {
        list: false,
        watch: false,
        auto_watch: false,
        debug: false,
        task: Some(String::from("deploy")),
        subtask: Some(String::from("prod")),
        args: vec![],
    };
    let mut manager = GraphManager::new();
    manager.initialize().unwrap();
    let result = get_task_name(&args, &manager).unwrap();
    assert_eq!(result, "deploy prod");
}

#[test]
fn test_get_task_name_with_task_only() {
    let args = Args {
        list: false,
        watch: false,
        auto_watch: false,
        debug: false,
        task: Some(String::from("test")),
        subtask: None,
        args: vec![],
    };
    let mut manager = GraphManager::new();
    manager.initialize().unwrap();
    let result = get_task_name(&args, &manager).unwrap();
    assert_eq!(result, "test");
}

#[test]
fn test_get_task_name_default_task_exists() {
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
    manager.initialize().unwrap();
    // Assuming that the default task exists
    let result = get_task_name(&args, &manager).unwrap();
    assert_eq!(result, "default");
}

#[test]
fn test_get_task_name_default_task_not_exists() {
    let args = Args {
        list: false,
        watch: false,
        auto_watch: false,
        debug: false,
        task: None,
        subtask: None,
        args: vec![],
    };
    let manager = GraphManager::new(); // empty manager, no tasks
    let result = get_task_name(&args, &manager);
    assert!(matches!(result, Err(BodoError::NoTaskSpecified)));
}

#[test]
fn test_get_task_name_task_not_found() {
    let args = Args {
        list: false,
        watch: false,
        auto_watch: false,
        debug: false,
        task: Some(String::from("non_existent_task")),
        subtask: None,
        args: vec![],
    };
    let mut manager = GraphManager::new();
    manager.initialize().unwrap();
    let result = get_task_name(&args, &manager);
    assert!(matches!(result, Err(BodoError::TaskNotFound(_))));
}
