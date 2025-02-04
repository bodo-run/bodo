use bodo::cli::{get_task_name, Args};
use bodo::errors::BodoError;
use bodo::graph::{Node, NodeKind, TaskData};
use bodo::manager::GraphManager;
use std::collections::HashMap;

#[cfg(test)]
mod new_tests {
    use super::*;

    #[test]
    fn test_get_task_name_with_subtask_exists() {
        let mut manager = GraphManager::new();
        // Add a task with concatenated name "build unit"
        manager.graph.nodes.push(Node {
            id: 0,
            kind: NodeKind::Task(TaskData {
                name: "build unit".to_string(),
                description: Some("Deploy production".to_string()),
                command: Some("echo deploy prod".to_string()),
                working_dir: None,
                env: HashMap::new(),
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
            metadata: HashMap::new(),
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
        };
        let result = get_task_name(&args, &manager).unwrap();
        assert_eq!(result, "build unit");
    }

    #[test]
    fn test_get_task_name_with_subtask_not_found() {
        let mut manager = GraphManager::new();
        // Only add task "build" exists.
        manager.graph.nodes.push(Node {
            id: 0,
            kind: NodeKind::Task(TaskData {
                name: "build".to_string(),
                description: Some("build task".to_string()),
                command: Some("echo build".to_string()),
                working_dir: None,
                env: HashMap::new(),
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
            metadata: HashMap::new(),
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
        };
        let result = get_task_name(&args, &manager);
        assert!(matches!(result, Err(BodoError::TaskNotFound(_))));
    }
}
