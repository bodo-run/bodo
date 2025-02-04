use bodo::cli::Args;
use bodo::config::{BodoConfig, TaskArgument};
use bodo::errors::BodoError;
use bodo::graph::{Graph, NodeKind, TaskData};
use bodo::manager::GraphManager;
use bodo::plugins::prefix_plugin::PrefixPlugin;
use bodo::process::{color_line, parse_color};
use std::collections::HashMap;

#[cfg(test)]
mod new_tests {
    use super::*;

    #[test]
    fn test_cli_get_task_name_default_exists() {
        let mut manager = GraphManager::new();
        // Manually add default task to graph and registry:
        manager.graph.nodes.push(bodo::graph::Node {
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
        };
        let name = super::super::super::cli::get_task_name(&args, &manager).unwrap();
        assert_eq!(name, "default");
    }

    #[test]
    fn test_graph_detect_cycle_none() {
        let mut graph = Graph::new();
        let _ = graph.add_node(NodeKind::Task(TaskData {
            name: "a".to_string(),
            description: None,
            command: Some("echo a".to_string()),
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
        assert!(graph.detect_cycle().is_none());
    }

    #[test]
    fn test_graph_detect_cycle_some() {
        let mut graph = Graph::new();
        let id1 = graph.add_node(NodeKind::Task(TaskData {
            name: "a".to_string(),
            description: None,
            command: Some("echo a".to_string()),
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
        let id2 = graph.add_node(NodeKind::Task(TaskData {
            name: "b".to_string(),
            description: None,
            command: Some("echo b".to_string()),
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
        graph.add_edge(id1, id2).unwrap();
        graph.add_edge(id2, id1).unwrap();
        let cycle = graph.detect_cycle();
        assert!(cycle.is_some());
    }

    #[test]
    fn test_graph_topological_sort_order() -> crate::Result<()> {
        let mut graph = Graph::new();
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
