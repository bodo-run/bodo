use bodo::cli::{get_task_name, Args};
use bodo::config::{validate_task_name, BodoConfig, TaskArgument, TaskConfig, WatchConfig};
use bodo::errors::BodoError;
use bodo::graph::{CommandData, ConcurrentGroupData, Graph, Node, NodeKind, TaskData};
use bodo::plugins::concurrent_plugin::ConcurrentPlugin;
use bodo::plugins::execution_plugin::ExecutionPlugin;
use bodo::plugins::prefix_plugin::PrefixPlugin;
use bodo::plugins::timeout_plugin::TimeoutPlugin;
use bodo::plugins::watch_plugin::WatchPlugin;
use bodo::Plugin;
use std::collections::HashMap;

#[cfg(test)]
mod new_tests {
    use super::*;

    #[test]
    fn test_cli_get_task_name_default_exists() {
        let mut manager = bodo::manager::GraphManager::new();
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
        };
        let name = get_task_name(&args, &manager).unwrap();
        assert_eq!(name, "default");
    }

    #[test]
    fn test_cli_get_task_name_with_existing_task() {
        let mut manager = bodo::manager::GraphManager::new();
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
        };
        let name = get_task_name(&args, &manager).unwrap();
        assert_eq!(name, "build");
    }

    #[test]
    fn test_bodo_error_variants_display() {
        let io_err = BodoError::IoError(std::io::Error::new(std::io::ErrorKind::Other, "io error"));
        assert_eq!(format!("{}", io_err), "io error");

        let watcher_err = BodoError::WatcherError("watcher error".to_string());
        assert_eq!(format!("{}", watcher_err), "watcher error");

        let task_not_found = BodoError::TaskNotFound("not_found".to_string());
        assert_eq!(format!("{}", task_not_found), "not found");

        let plugin_err = BodoError::PluginError("plugin fail".to_string());
        assert_eq!(format!("{}", plugin_err), "Plugin error: plugin fail");

        let no_task = BodoError::NoTaskSpecified;
        assert_eq!(
            format!("{}", no_task),
            "No task specified and no scripts/script.yaml found"
        );

        let validation_err = BodoError::ValidationError("val error".to_string());
        assert_eq!(format!("{}", validation_err), "Validation error: val error");
    }

    #[test]
    fn test_prefix_plugin_next_color_cycle() {
        let mut plugin = PrefixPlugin::new();
        let mut colors = Vec::new();
        // Call next_color 10 times; DEFAULT_COLORS (6 items) will cycle.
        for _ in 0..10 {
            colors.push(plugin.next_color());
        }
        assert_eq!(colors.len(), 10);
        // Check that the first 6 are distinct.
        let unique: std::collections::HashSet<_> = colors.iter().take(6).collect();
        assert_eq!(unique.len(), 6);
    }

    #[test]
    fn test_color_line_function() {
        let prefix = "TEST";
        let prefix_color = Some("red".to_string());
        let line = "Hello";
        let colored = bodo::process::color_line(prefix, &prefix_color, line, false);
        // Check that the returned string contains the prefix (in brackets) and the line.
        assert!(colored.contains("[TEST]"));
        assert!(colored.contains("Hello"));
    }

    #[test]
    fn test_parse_color_invalid() {
        // Test that parse_color returns None for an unknown color string.
        assert_eq!(bodo::process::parse_color("unknowncolor"), None);
    }

    #[test]
    fn test_manager_build_graph_empty() {
        let mut manager = bodo::manager::GraphManager::new();
        let config = BodoConfig::default();
        // build_graph currently returns an empty graph.
        let graph = manager.build_graph(config).unwrap();
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
        assert!(graph.task_registry.is_empty());
    }

    #[test]
    fn test_manager_apply_task_arguments_success() {
        let mut manager = bodo::manager::GraphManager::new();
        // Manually add a task with an argument that has a default.
        let task = TaskData {
            name: "greet".to_string(),
            description: Some("Greet Task".to_string()),
            command: Some("echo $GREETING".to_string()),
            working_dir: None,
            env: HashMap::new(),
            exec_paths: vec![],
            arguments: vec![TaskArgument {
                name: "GREETING".to_string(),
                description: Some("Greeting msg".to_string()),
                required: true,
                default: Some("Hello".to_string()),
            }],
            is_default: false,
            script_id: "".to_string(),
            script_display_name: "".to_string(),
            watch: None,
            pre_deps: vec![],
            post_deps: vec![],
            concurrently: vec![],
            concurrently_options: Default::default(),
        };

        manager.graph.nodes.push(Node {
            id: 0,
            kind: NodeKind::Task(task),
            metadata: HashMap::new(),
        });
        manager.graph.task_registry.insert("greet".to_string(), 0);
        let result = manager.apply_task_arguments("greet", &[]);
        assert!(result.is_ok());
        if let NodeKind::Task(ref task_data) = manager.graph.nodes[0].kind {
            assert_eq!(task_data.env.get("GREETING"), Some(&"Hello".to_string()));
        } else {
            panic!("Expected task node");
        }
    }

    #[test]
    fn test_manager_apply_task_arguments_failure() {
        let mut manager = bodo::manager::GraphManager::new();
        // Add a task with a required argument that has no default.
        let task = TaskData {
            name: "greet".to_string(),
            description: None,
            command: Some("echo $NAME".to_string()),
            working_dir: None,
            env: HashMap::new(),
            exec_paths: vec![],
            arguments: vec![TaskArgument {
                name: "NAME".to_string(),
                description: None,
                required: true,
                default: None,
            }],
            is_default: false,
            script_id: "".to_string(),
            script_display_name: "".to_string(),
            watch: None,
            pre_deps: vec![],
            post_deps: vec![],
            concurrently: vec![],
            concurrently_options: Default::default(),
        };

        manager.graph.nodes.push(Node {
            id: 0,
            kind: NodeKind::Task(task),
            metadata: HashMap::new(),
        });
        manager.graph.task_registry.insert("greet".to_string(), 0);
        let result = manager.apply_task_arguments("greet", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_concurrent_plugin() {
        let mut plugin = ConcurrentPlugin::new();

        let mut graph = Graph::new();

        // Create tasks
        let task_data_main = TaskData {
            name: "main_task".to_string(),
            description: None,
            command: None, // No command, will have concurrent tasks
            working_dir: None,
            env: HashMap::new(),
            exec_paths: vec![],
            arguments: vec![],
            concurrently: vec![],
            pre_deps: vec![],
            post_deps: vec![],
            concurrently_options: Default::default(),
            is_default: true,
            script_id: "script".to_string(),
            script_display_name: "script".to_string(),
            watch: None,
        };

        let main_task_id = graph.add_node(NodeKind::Task(task_data_main));

        let task_data_child1 = TaskData {
            name: "child_task1".to_string(),
            description: None,
            command: Some("echo Child 1".to_string()),
            working_dir: None,
            env: HashMap::new(),
            exec_paths: vec![],
            arguments: vec![],
            concurrently: vec![],
            pre_deps: vec![],
            post_deps: vec![],
            concurrently_options: Default::default(),
            is_default: false,
            script_id: "script".to_string(),
            script_display_name: "script".to_string(),
            watch: None,
        };

        let child1_id = graph.add_node(NodeKind::Task(task_data_child1));

        let task_data_child2 = TaskData {
            name: "child_task2".to_string(),
            description: None,
            command: Some("echo Child 2".to_string()),
            working_dir: None,
            env: HashMap::new(),
            exec_paths: vec![],
            arguments: vec![],
            concurrently: vec![],
            pre_deps: vec![],
            post_deps: vec![],
            concurrently_options: Default::default(),
            is_default: false,
            script_id: "script".to_string(),
            script_display_name: "script".to_string(),
            watch: None,
        };

        let child2_id = graph.add_node(NodeKind::Task(task_data_child2));

        // Register child tasks in task_registry
        graph
            .task_registry
            .insert("child_task1".to_string(), child1_id);
        graph
            .task_registry
            .insert("child_task2".to_string(), child2_id);

        // Set up the main_task to have concurrent tasks
        let main_node = &mut graph.nodes[main_task_id as usize];
        // Set the metadata 'concurrently' directly as a JSON array string
        main_node.metadata.insert(
            "concurrently".to_string(),
            "[\"child_task1\", \"child_task2\"]".to_string(),
        );
        // Also set 'fail_fast' and 'max_concurrent' in metadata
        main_node
            .metadata
            .insert("fail_fast".to_string(), "true".to_string());
        main_node
            .metadata
            .insert("max_concurrent".to_string(), "2".to_string());

        // Apply the plugin
        let result = plugin.on_graph_build(&mut graph);
        assert!(
            result.is_ok(),
            "Plugin on_graph_build returned an error: {:?}",
            result.unwrap_err()
        );

        // Check that a ConcurrentGroup node has been added
        let group_nodes: Vec<_> = graph
            .nodes
            .iter()
            .filter_map(|node| {
                if let NodeKind::ConcurrentGroup(group_data) = &node.kind {
                    Some((node.id, group_data))
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(group_nodes.len(), 1, "Expected one concurrent group node");

        let (_group_id, group_data) = &group_nodes[0];
        assert_eq!(group_data.child_nodes.len(), 2);
        assert!(group_data.child_nodes.contains(&child1_id));
        assert!(group_data.child_nodes.contains(&child2_id));

        // Check the 'fail_fast' and 'max_concurrent' settings
        assert!(group_data.fail_fast, "Expected fail_fast to be true");
        assert_eq!(
            group_data.max_concurrent,
            Some(2),
            "Expected max_concurrent to be 2"
        );

        // Check that edges have been added appropriately
        // Edge from main_task to group
        assert!(graph
            .edges
            .iter()
            .any(|edge| edge.from == main_task_id && edge.to == group_nodes[0].0));

        // Edges from group to child tasks
        assert!(graph
            .edges
            .iter()
            .any(|edge| edge.from == group_nodes[0].0 && edge.to == child1_id));
        assert!(graph
            .edges
            .iter()
            .any(|edge| edge.from == group_nodes[0].0 && edge.to == child2_id));
    }

    #[test]
    fn test_concurrent_plugin_with_commands() {
        let mut plugin = ConcurrentPlugin::new();

        let mut graph = Graph::new();

        // Create a main task
        let task_data_main = TaskData {
            name: "main_task".to_string(),
            description: None,
            command: None, // No command, will have concurrent commands
            working_dir: None,
            env: HashMap::new(),
            exec_paths: vec![],
            arguments: vec![],
            concurrently: vec![],
            pre_deps: vec![],
            post_deps: vec![],
            concurrently_options: Default::default(),
            is_default: true,
            script_id: "script".to_string(),
            script_display_name: "script".to_string(),
            watch: None,
        };

        let main_task_id = graph.add_node(NodeKind::Task(task_data_main));

        // Set up the main_task to have concurrent commands
        let main_node = &mut graph.nodes[main_task_id as usize];
        // Set the metadata 'concurrently' directly
        main_node.metadata.insert(
            "concurrently".to_string(),
            r#"[{"command": "echo Command 1"}, {"command": "echo Command 2"}]"#.to_string(),
        );

        // Apply the plugin
        let result = plugin.on_graph_build(&mut graph);
        assert!(
            result.is_ok(),
            "Plugin on_graph_build returned an error: {:?}",
            result.unwrap_err()
        );

        // Check that a ConcurrentGroup node has been added
        let group_nodes: Vec<_> = graph
            .nodes
            .iter()
            .filter_map(|node| {
                if let NodeKind::ConcurrentGroup(group_data) = &node.kind {
                    Some((node.id, group_data))
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(group_nodes.len(), 1, "Expected one concurrent group node");

        let (_group_id, group_data) = &group_nodes[0];
        assert_eq!(group_data.child_nodes.len(), 2);

        // The child nodes should be Command nodes
        for &child_id in &group_data.child_nodes {
            let child_node = &graph.nodes[child_id as usize];
            if let NodeKind::Command(cmd_data) = &child_node.kind {
                assert!(
                    cmd_data.raw_command == "echo Command 1"
                        || cmd_data.raw_command == "echo Command 2"
                );
            } else {
                panic!("Expected Command node");
            }
        }
    }

    #[test]
    fn test_concurrent_plugin_nonexistent_task() {
        let mut plugin = ConcurrentPlugin::new();

        let mut graph = Graph::new();

        // Create a main task
        let task_data_main = TaskData {
            name: "main_task".to_string(),
            description: None,
            command: None,
            working_dir: None,
            env: HashMap::new(),
            exec_paths: vec![],
            arguments: vec![],
            concurrently: vec![],
            pre_deps: vec![],
            post_deps: vec![],
            concurrently_options: Default::default(),
            is_default: true,
            script_id: "script".to_string(),
            script_display_name: "script".to_string(),
            watch: None,
        };

        let main_task_id = graph.add_node(NodeKind::Task(task_data_main));

        // Set up the main_task to have a nonexistent concurrent task
        let main_node = &mut graph.nodes[main_task_id as usize];
        main_node.metadata.insert(
            "concurrently".to_string(),
            "[\"nonexistent_task\"]".to_string(),
        );

        // Apply the plugin
        let result = plugin.on_graph_build(&mut graph);
        assert!(
            result.is_err(),
            "Expected error due to nonexistent task, but got success"
        );
        let error = result.unwrap_err();
        assert!(
            matches!(error, BodoError::PluginError(_)),
            "Expected PluginError, got {:?}",
            error
        );
    }

    #[test]
    fn test_concurrent_plugin_invalid_dependency_format() {
        let mut plugin = ConcurrentPlugin::new();

        let mut graph = Graph::new();

        // Create a main task
        let task_data_main = TaskData {
            name: "main_task".to_string(),
            description: None,
            command: None,
            working_dir: None,
            env: HashMap::new(),
            exec_paths: vec![],
            arguments: vec![],
            concurrently: vec![],
            pre_deps: vec![],
            post_deps: vec![],
            concurrently_options: Default::default(),
            is_default: true,
            script_id: "script".to_string(),
            script_display_name: "script".to_string(),
            watch: None,
        };

        let main_task_id = graph.add_node(NodeKind::Task(task_data_main));
        let main_node = &mut graph.nodes[main_task_id as usize];
        main_node.metadata.insert(
            "concurrently".to_string(),
            "[123, true]".to_string(), // Invalid format
        );

        let result = plugin.on_graph_build(&mut graph);
        assert!(
            result.is_err(),
            "Expected error due to invalid dependency format, but got success"
        );
        let error = result.unwrap_err();
        assert!(
            matches!(error, BodoError::PluginError(_)),
            "Expected PluginError, got {:?}",
            error
        );
    }

    #[test]
    fn test_concurrent_plugin_empty_concurrently() {
        let mut plugin = ConcurrentPlugin::new();

        let mut graph = Graph::new();

        // Create a main task
        let task_data_main = TaskData {
            name: "main_task".to_string(),
            description: None,
            command: None,
            working_dir: None,
            env: HashMap::new(),
            exec_paths: vec![],
            arguments: vec![],
            concurrently: vec![],
            pre_deps: vec![],
            post_deps: vec![],
            concurrently_options: Default::default(),
            is_default: true,
            script_id: "script".to_string(),
            script_display_name: "script".to_string(),
            watch: None,
        };

        let main_task_id = graph.add_node(NodeKind::Task(task_data_main));

        // Set up the main_task with empty 'concurrently' metadata
        let main_node = &mut graph.nodes[main_task_id as usize];
        main_node
            .metadata
            .insert("concurrently".to_string(), "[]".to_string());

        // Apply the plugin
        let result = plugin.on_graph_build(&mut graph);
        assert!(
            result.is_ok(),
            "Plugin on_graph_build returned an error: {:?}",
            result.unwrap_err()
        );

        // Check that a ConcurrentGroup node has been added with no children
        let group_nodes: Vec<_> = graph
            .nodes
            .iter()
            .filter_map(|node| {
                if let NodeKind::ConcurrentGroup(group_data) = &node.kind {
                    Some((node.id, group_data))
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(group_nodes.len(), 1, "Expected one concurrent group node");

        let (_group_id, group_data) = &group_nodes[0];
        assert_eq!(
            group_data.child_nodes.len(),
            0,
            "Expected no child nodes in the group"
        );
    }

    #[test]
    fn test_concurrent_plugin_nonexistent_task_in_object() {
        let mut plugin = ConcurrentPlugin::new();
        let mut graph = Graph::new();

        let task_data_main = TaskData {
            name: "main_task".to_string(),
            description: None,
            command: None,
            working_dir: None,
            env: HashMap::new(),
            exec_paths: vec![],
            arguments: vec![],
            concurrently: vec![],
            pre_deps: vec![],
            post_deps: vec![],
            concurrently_options: Default::default(),
            is_default: true,
            script_id: "script".to_string(),
            script_display_name: "script".to_string(),
            watch: None,
        };
        let main_task_id = graph.add_node(NodeKind::Task(task_data_main));
        let main_node = &mut graph.nodes[main_task_id as usize];
        main_node.metadata.insert(
            "concurrently".to_string(),
            serde_json::to_string(&json!([{"task": "nonexistent"}])).unwrap(),
        );
        let result = plugin.on_graph_build(&mut graph);
        assert!(
            result.is_err(),
            "Expected error for nonexistent task in object dependency"
        );
    }

    #[test]
    fn test_concurrent_plugin_invalid_dependency_format_again() {
        let mut plugin = ConcurrentPlugin::new();
        let mut graph = Graph::new();

        let task_data_main = TaskData {
            name: "main_task".to_string(),
            description: None,
            command: None,
            working_dir: None,
            env: HashMap::new(),
            exec_paths: vec![],
            arguments: vec![],
            concurrently: vec![],
            pre_deps: vec![],
            post_deps: vec![],
            concurrently_options: Default::default(),
            is_default: true,
            script_id: "script".to_string(),
            script_display_name: "script".to_string(),
            watch: None,
        };

        let main_task_id = graph.add_node(NodeKind::Task(task_data_main));
        let main_node = &mut graph.nodes[main_task_id as usize];
        main_node.metadata.insert(
            "concurrently".to_string(),
            "[123, true]".to_string(), // Invalid format
        );

        let result = plugin.on_graph_build(&mut graph);
        assert!(
            result.is_err(),
            "Expected error due to invalid dependency format, but got success"
        );
        let error = result.unwrap_err();
        assert!(
            matches!(error, BodoError::PluginError(_)),
            "Expected PluginError, got {:?}",
            error
        );
    }

    #[test]
    fn test_watch_plugin_create_watcher_test() {
        let (watcher, rx) = WatchPlugin::create_watcher_test().expect("Failed to create watcher");
        // Expect timeout since no events occur.
        match rx.recv_timeout(std::time::Duration::from_millis(100)) {
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => assert!(true),
            _ => panic!("Expected timeout when no events occur"),
        }
        drop(watcher);
    }
}
