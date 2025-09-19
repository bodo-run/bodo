use bodo::cli::{get_task_name, Args};
use bodo::errors::BodoError;
use bodo::graph::{Node, NodeKind, TaskData};
use bodo::manager::GraphManager;
use bodo::plugins::prefix_plugin::PrefixPlugin;
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
            dry_run: false,
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
            dry_run: false,
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
}
