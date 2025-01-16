use crate::{
    errors::Result,
    graph::{Graph, NodeKind},
    Plugin, PluginConfig,
};
use async_trait::async_trait;
use std::any::Any;

pub struct PathPlugin {
    default_paths: Vec<String>,
}

#[async_trait]
impl Plugin for PathPlugin {
    fn name(&self) -> &'static str {
        "path"
    }

    async fn on_init(&mut self, config: &PluginConfig) -> Result<()> {
        if let Some(options) = &config.options {
            if let Some(paths) = options.get("default_paths") {
                if let Some(paths) = paths.as_array() {
                    self.default_paths = paths
                        .iter()
                        .filter_map(|p| p.as_str().map(|s| s.to_string()))
                        .collect();
                }
            }
        }
        Ok(())
    }

    async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        for node in &mut graph.nodes {
            let working_dir = match &node.kind {
                NodeKind::Task(task_data) => task_data.working_dir.clone(),
                NodeKind::Command(cmd_data) => cmd_data.working_dir.clone(),
                _ => None,
            };

            let mut paths = self.default_paths.clone();
            if let Some(dir) = working_dir {
                paths.push(dir);
            }

            if !paths.is_empty() {
                node.metadata
                    .insert("env.PATH".to_string(), paths.join(":"));
            }
        }
        Ok(())
    }

    fn on_task_start(&mut self) {
        // Nothing to do
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl PathPlugin {
    pub fn new() -> Self {
        Self {
            default_paths: Vec::new(),
        }
    }
}
