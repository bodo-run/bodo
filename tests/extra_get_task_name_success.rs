use bodo::cli::{get_task_name, Args};
use bodo::errors::BodoError;
use bodo::graph::{Node, NodeKind, TaskData};
use bodo::manager::GraphManager;
use std::collections::HashMap;

#[cfg(test)]
mod extra_get_task_name_success {
    use super::*;

    #[test]
    fn test_success_default() {
        let mut gm = GraphManager::new();
        // Add default task to the graph.
        gm.graph.nodes.push(Node {
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
        gm.graph.task_registry.insert("default".to_string(), 0);

        let args = Args {
            list: false,
            watch: false,
            auto_watch: false,
            debug: false,
            verbose: false,
            quiet: false,
            task: None,
            subtask: None,
            args: vec![],
        };
        let res = get_task_name(&args, &gm);
        assert_eq!(res.unwrap(), "default");
    }

    #[test]
    fn test_success_with_task() {
        let mut gm = GraphManager::new();
        gm.graph.nodes.push(Node {
            id: 0,
            kind: NodeKind::Task(TaskData {
                name: "build".to_string(),
                description: Some("Build Task".to_string()),
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
        gm.graph.task_registry.insert("build".to_string(), 0);

        let args = Args {
            list: false,
            watch: false,
            auto_watch: false,
            debug: false,
            verbose: false,
            quiet: false,
            task: Some("build".to_string()),
            subtask: None,
            args: vec![],
        };
        let res = get_task_name(&args, &gm);
        assert_eq!(res.unwrap(), "build");
    }

    #[test]
    fn test_success_with_subtask() {
        let mut gm = GraphManager::new();
        gm.graph.nodes.push(Node {
            id: 0,
            kind: NodeKind::Task(TaskData {
                name: "deploy prod".to_string(),
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
        gm.graph.task_registry.insert("deploy prod".to_string(), 0);

        let args = Args {
            list: false,
            watch: false,
            auto_watch: false,
            debug: false,
            verbose: false,
            quiet: false,
            task: Some("deploy".to_string()),
            subtask: Some("prod".to_string()),
            args: vec![],
        };
        let res = get_task_name(&args, &gm);
        assert_eq!(res.unwrap(), "deploy prod");
    }

    #[test]
    fn test_no_default_error() {
        let gm = GraphManager::new();
        let args = Args {
            list: false,
            watch: false,
            auto_watch: false,
            debug: false,
            verbose: false,
            quiet: false,
            task: None,
            subtask: None,
            args: vec![],
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
            verbose: false,
            quiet: false,
            task: Some("nonexistent".to_string()),
            subtask: None,
            args: vec![],
        };
        let res = get_task_name(&args, &gm);
        assert!(matches!(res, Err(BodoError::TaskNotFound(_))));
    }
}
