use crate::config::{BodoConfig, TaskConfig};
use crate::graph::TaskGraph;
use std::error::Error;

pub trait BodoPlugin {
    fn on_bodo_init(&mut self, _config: &mut BodoConfig) {}
    fn on_task_graph_construct_start(&mut self, _tasks: &mut [TaskConfig]) {}
    fn on_task_graph_construct_end(&mut self, _graph: &TaskGraph) {}
    fn on_resolve_command(&mut self, _task: &mut TaskConfig) {}
    fn on_command_ready(&mut self, _command: &str, _task_name: &str) {}
    fn on_before_run(&mut self, _task_name: &str) {}
    fn on_after_run(&mut self, _task_name: &str, _status_code: i32) {}
    fn on_error(&mut self, _task_name: &str, _err: &dyn Error) {}
    fn on_before_watch(&mut self, _patterns: &mut Vec<String>) {}
    fn on_after_watch_event(&mut self, _changed_file: &str) {}
    fn on_bodo_exit(&mut self, _exit_code: i32) {}
}

pub struct PluginManager {
    config: BodoConfig,
    plugins: Vec<Box<dyn BodoPlugin>>,
}

impl PluginManager {
    pub fn new(config: BodoConfig) -> Self {
        Self {
            config,
            plugins: vec![],
        }
    }

    pub fn register_plugin(&mut self, plugin: Box<dyn BodoPlugin>) {
        self.plugins.push(plugin);
    }

    pub fn on_bodo_init(&mut self) {
        for plugin in &mut self.plugins {
            plugin.on_bodo_init(&mut self.config);
        }
    }

    pub fn on_task_graph_construct_start(&mut self, tasks: &mut [TaskConfig]) {
        for plugin in &mut self.plugins {
            plugin.on_task_graph_construct_start(tasks);
        }
    }

    pub fn on_task_graph_construct_end(&mut self, graph: &TaskGraph) {
        for plugin in &mut self.plugins {
            plugin.on_task_graph_construct_end(graph);
        }
    }

    pub fn on_resolve_command(&mut self, task: &mut TaskConfig) {
        for plugin in &mut self.plugins {
            plugin.on_resolve_command(task);
        }
    }

    pub fn on_command_ready(&mut self, command: &str, task_name: &str) {
        for plugin in &mut self.plugins {
            plugin.on_command_ready(command, task_name);
        }
    }

    pub fn on_before_run(&mut self, task_name: &str) {
        for plugin in &mut self.plugins {
            plugin.on_before_run(task_name);
        }
    }

    pub fn on_after_run(&mut self, task_name: &str, status_code: i32) {
        for plugin in &mut self.plugins {
            plugin.on_after_run(task_name, status_code);
        }
    }

    pub fn on_error(&mut self, task_name: &str, err: &dyn Error) {
        for plugin in &mut self.plugins {
            plugin.on_error(task_name, err);
        }
    }

    pub fn on_before_watch(&mut self, patterns: &mut Vec<String>) {
        for plugin in &mut self.plugins {
            plugin.on_before_watch(patterns);
        }
    }

    pub fn on_after_watch_event(&mut self, changed_file: &str) {
        for plugin in &mut self.plugins {
            plugin.on_after_watch_event(changed_file);
        }
    }

    pub fn on_bodo_exit(&mut self, exit_code: i32) {
        for plugin in &mut self.plugins {
            plugin.on_bodo_exit(exit_code);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin;

    impl BodoPlugin for TestPlugin {
        fn on_before_run(&mut self, task_name: &str) {
            println!("Test plugin: before run {}", task_name);
        }
    }

    #[test]
    fn test_plugin_manager_creation() {
        let config = BodoConfig::default();
        let plugin_manager = PluginManager::new(config);
        assert!(plugin_manager.plugins.is_empty());
    }

    #[test]
    fn test_plugin_registration() {
        let config = BodoConfig::default();
        let mut plugin_manager = PluginManager::new(config);
        plugin_manager.register_plugin(Box::new(TestPlugin));
        assert_eq!(plugin_manager.plugins.len(), 1);
    }
}
