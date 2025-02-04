use bodo::cli::{get_task_name, Args};
use bodo::config::{BodoConfig, TaskArgument};
use bodo::errors::BodoError;
use bodo::graph::{Node, NodeKind, TaskData};
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
        let colored = color_line(prefix, &prefix_color, line, false);
        // Check that the returned string contains the prefix (in brackets) and the line.
        assert!(colored.contains("[TEST]"));
        assert!(colored.contains("Hello"));
    }

    #[test]
    fn test_parse_color_invalid() {
        // Test that parse_color returns None for an unknown color string.
        assert_eq!(parse_color("unknowncolor"), None);
    }

    #[test]
    fn test_manager_build_graph_empty() {
        let mut manager = GraphManager::new();
        let config = BodoConfig::default();
        // build_graph currently returns an empty graph.
        let graph = manager.build_graph(config).unwrap();
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
        assert!(graph.task_registry.is_empty());
    }

    #[test]
    fn test_manager_apply_task_arguments_success() {
        let mut manager = GraphManager::new();
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
        let mut manager = GraphManager::new();
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
}
