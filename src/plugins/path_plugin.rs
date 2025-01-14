use async_trait::async_trait;
use std::path::PathBuf;

use crate::{
    errors::Result,
    graph::{Graph, NodeKind},
    plugin::{Plugin, PluginConfig},
};

/// PathPlugin merges any "exec_paths" from the tasks/scripts into the node's environment PATH.
pub struct PathPlugin {
    pub default_paths: Vec<PathBuf>,
}

impl PathPlugin {
    pub fn new() -> Self {
        PathPlugin {
            default_paths: vec![],
        }
    }
}

#[async_trait]
impl Plugin for PathPlugin {
    fn name(&self) -> &'static str {
        "PathPlugin"
    }

    async fn on_init(&mut self, config: &PluginConfig) -> Result<()> {
        if let Some(options) = &config.options {
            if let Some(paths) = options.get("default_paths") {
                if let Some(paths) = paths.as_array() {
                    self.default_paths = paths
                        .iter()
                        .filter_map(|p| p.as_str().map(PathBuf::from))
                        .collect();
                }
            }
        }
        Ok(())
    }

    async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        for node in &mut graph.nodes {
            let mut paths = self.default_paths.clone();

            // Add any node-specific paths from metadata
            if let Some(node_paths) = node.metadata.get("exec_paths") {
                if let Ok(mut extra_paths) = serde_json::from_str::<Vec<PathBuf>>(node_paths) {
                    paths.append(&mut extra_paths);
                }
            }

            // Add working directory if specified
            match &node.kind {
                NodeKind::Task(task) => {
                    if let Some(dir) = &task.working_dir {
                        paths.push(dir.into());
                    }
                }
                NodeKind::Command(cmd) => {
                    if let Some(dir) = &cmd.working_dir {
                        paths.push(dir.into());
                    }
                }
            }

            // Convert paths to string and join with OS path separator
            let path_str = paths
                .iter()
                .filter_map(|p| p.to_str())
                .collect::<Vec<_>>()
                .join(&std::env::var("PATH_SEPARATOR").unwrap_or_else(|_| String::from(":")));

            // Set the PATH environment variable in metadata
            node.metadata.insert("env.PATH".to_string(), path_str);
        }
        Ok(())
    }
}
