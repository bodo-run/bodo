use async_trait::async_trait;
use std::any::Any;

use crate::{
    graph::{Graph, NodeKind},
    plugin::{Plugin, PluginConfig},
    Result,
};

pub struct PathPlugin {
    default_paths: Vec<String>,
}

impl PathPlugin {
    pub fn new() -> Self {
        Self {
            default_paths: Vec::new(),
        }
    }
}

impl Default for PathPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for PathPlugin {
    fn name(&self) -> &'static str {
        "PathPlugin"
    }

    fn priority(&self) -> i32 {
        85 // After env but before concurrency
    }

    async fn on_init(&mut self, config: &PluginConfig) -> Result<()> {
        if let Some(options) = &config.options {
            if let Some(paths) = options.get("default_paths") {
                if let Some(arr) = paths.as_array() {
                    self.default_paths = arr
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                }
            }
        }
        Ok(())
    }

    async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        for node in &mut graph.nodes {
            match &mut node.kind {
                NodeKind::Task(task_data) => {
                    let mut paths = self.default_paths.clone();
                    if let Some(cwd) = &task_data.working_dir {
                        paths.push(cwd.clone());
                    }
                    if !paths.is_empty() {
                        let path_str = paths.join(":");
                        task_data.env.insert("PATH".to_string(), path_str);
                    }
                }
                NodeKind::Command(cmd_data) => {
                    let mut paths = self.default_paths.clone();
                    if let Some(cwd) = &cmd_data.working_dir {
                        paths.push(cwd.clone());
                    }
                    if !paths.is_empty() {
                        let path_str = paths.join(":");
                        cmd_data.env.insert("PATH".to_string(), path_str);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
