use std::any::Any;
use std::env;

use crate::{
    graph::{Graph, NodeKind},
    plugin::{Plugin, PluginConfig},
    Result,
};

pub struct PathPlugin {
    default_paths: Vec<String>,
    preserve_path: bool,
}

impl PathPlugin {
    pub fn new() -> Self {
        Self {
            default_paths: Vec::new(),
            preserve_path: true,
        }
    }

    fn build_path(&self, working_dir: Option<&String>, exec_paths: &[String]) -> String {
        let mut paths = Vec::new();

        // Add working directory first if present
        if let Some(cwd) = working_dir {
            paths.push(cwd.clone());
        }

        // Add default paths from plugin config
        paths.extend(self.default_paths.iter().cloned());

        // Add user-specified exec_paths
        paths.extend(exec_paths.iter().cloned());

        // Optionally preserve existing PATH
        if self.preserve_path {
            if let Ok(current_path) = env::var("PATH") {
                paths.extend(current_path.split(':').map(String::from));
            }
        }

        paths.join(":")
    }

    // This function is added for testing purposes only.
    pub fn test_build_path(&self, working_dir: Option<&String>, exec_paths: &[String]) -> String {
        self.build_path(working_dir, exec_paths)
    }

    pub fn get_default_paths(&self) -> &Vec<String> {
        &self.default_paths
    }

    pub fn get_preserve_path(&self) -> bool {
        self.preserve_path
    }

    pub fn set_default_paths(&mut self, paths: Vec<String>) {
        self.default_paths = paths;
    }

    pub fn set_preserve_path(&mut self, preserve: bool) {
        self.preserve_path = preserve;
    }
}

impl Default for PathPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for PathPlugin {
    fn name(&self) -> &'static str {
        "PathPlugin"
    }

    fn priority(&self) -> i32 {
        85
    }

    fn on_init(&mut self, config: &PluginConfig) -> Result<()> {
        if let Some(options) = &config.options {
            if let Some(paths) = options.get("default_paths") {
                if let Some(arr) = paths.as_array() {
                    self.default_paths = arr
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();
                }
            }
            if let Some(preserve) = options.get("preserve_path") {
                if let Some(val) = preserve.as_bool() {
                    self.preserve_path = val;
                }
            }
        }
        Ok(())
    }

    fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        for node in &mut graph.nodes {
            match &mut node.kind {
                NodeKind::Task(task_data) => {
                    let path_str =
                        self.build_path(task_data.working_dir.as_ref(), &task_data.exec_paths);
                    if !path_str.is_empty() {
                        task_data.env.insert("PATH".to_string(), path_str);
                    }
                }
                NodeKind::Command(cmd_data) => {
                    let path_str = self.build_path(cmd_data.working_dir.as_ref(), &[]);
                    if !path_str.is_empty() {
                        cmd_data.env.insert("PATH".to_string(), path_str);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn on_after_run(&mut self, _graph: &mut Graph) -> Result<()> {
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
