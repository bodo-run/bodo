extern crate bodo;

use bodo::cli::{get_task_name, Args};
use bodo::errors::BodoError;
use bodo::graph::{Node, NodeKind, TaskData};
use bodo::manager::GraphManager;
use std::collections::HashMap;

#[cfg(test)]
mod new_tests {
    use super::*;

    #[test]
    fn test_cli_get_task_name_default_exists() {
        let mut manager = GraphManager::new();
        // Manually add default task to graph and registry:
        manager.graph.nodes.push(Node {
            id: 0,
            kind: NodeKind::Task(TaskData {
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
                pre_deps: vec![],
                post_deps: vec![],
                concurrently: vec![],
                concurrently_options: Default::default(),
            }),
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
            dry_run: false,
        };
        let name = get_task_name(&args, &manager).unwrap();
        assert_eq!(name, "default");
    }

    #[test]
    fn test_cli_get_task_name_with_existing_task() {
        let mut manager = GraphManager::new();
        // Add task "build"
        manager.graph.nodes.push(Node {
            id: 0,
            kind: NodeKind::Task(TaskData {
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
            subtask: None,
            args: vec![],
            dry_run: false,
        };
        let name = get_task_name(&args, &manager).unwrap();
        assert_eq!(name, "build");
    }

    #[test]
    fn test_no_default_error() {
        let gm = GraphManager::new();
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
        let res = get_task_name(&args, &gm);
        assert!(matches!(res, Err(BodoError::NoTaskSpecified)));
    }

    #[test]
    fn test_task_not_found_error() {
        let mut gm = GraphManager::new();
        gm.graph.nodes.push(Node {
            id: 0,
            kind: NodeKind::Task(TaskData {
                name: "existing".to_string(),
                description: None,
                command: Some("echo existing".to_string()),
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
        gm.graph.task_registry.insert("existing".to_string(), 0);

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
        let res = get_task_name(&args, &gm);
        assert!(matches!(res, Err(BodoError::TaskNotFound(_))));
    }
}
