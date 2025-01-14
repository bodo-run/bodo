use std::path::PathBuf;

use async_trait::async_trait;
use serde_json::Value;
use std::any::Any;

use crate::{
    errors::Result,
    graph::{CommandData, Graph, NodeKind, TaskData},
    plugin::{Plugin, PluginConfig},
};

/// PathPlugin merges any "exec_paths" from the tasks/scripts into the node's environment PATH.
#[derive(Default)]
pub struct PathPlugin {
    default_paths: Vec<PathBuf>,
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
            if let Some(Value::Array(paths)) = options.get("default_paths") {
                self.default_paths = paths
                    .iter()
                    .filter_map(|p| p.as_str().map(PathBuf::from))
                    .collect();
            }
        }
        Ok(())
    }

    async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        for node in graph.nodes.iter_mut() {
            let working_dir = match &node.kind {
                NodeKind::Task(TaskData { working_dir, .. }) => working_dir,
                NodeKind::Command(CommandData { working_dir, .. }) => working_dir,
            };

            let mut paths = self.default_paths.clone();
            if let Some(exec_paths) = node.metadata.get("exec_paths") {
                if let Ok(Value::Array(exec_paths)) = serde_json::from_str(exec_paths) {
                    paths.extend(
                        exec_paths
                            .iter()
                            .filter_map(|p| p.as_str().map(PathBuf::from)),
                    );
                }
            }

            if let Some(dir) = working_dir {
                paths.push(PathBuf::from(dir));
            }

            let path_str = paths
                .iter()
                .filter_map(|p| p.to_str())
                .collect::<Vec<_>>()
                .join(":");

            node.metadata.insert("env.PATH".to_string(), path_str);
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
