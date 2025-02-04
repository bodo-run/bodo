use bodo::cli::{get_task_name, Args};
use bodo::config::{BodoConfig, TaskArgument};
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
            no_watch: false,
            debug: false,
            task: None,
            subtask: None,
            args: vec![],
        };
        let name = get_task_name(&args, &manager).unwrap();
        assert_eq!(name, "default");
    }
}
