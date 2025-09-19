use bodo::cli::{get_task_name, Args};
use bodo::graph::{Node, NodeKind, TaskData};
use bodo::manager::GraphManager;
use bodo::Result;
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
            verbose: false,
            quiet: false,
            dry_run: false,
            task: None,
            subtask: None,
            args: vec![],
        };
        let name = get_task_name(&args, &manager).unwrap();
        assert_eq!(name, "default");
    }

    #[test]
    fn test_graph_topological_sort_order() -> Result<()> {
        let mut graph = GraphManager::new().graph;
        let a = graph.add_node(NodeKind::Task(TaskData {
            name: "A".to_string(),
            description: None,
            command: Some("echo A".to_string()),
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
        }));
        let b = graph.add_node(NodeKind::Task(TaskData {
            name: "B".to_string(),
            description: None,
            command: Some("echo B".to_string()),
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
        }));
        graph.add_edge(a, b).unwrap();
        let sorted = graph.topological_sort()?;
        assert_eq!(sorted, vec![a, b]);
        Ok(())
    }
}
