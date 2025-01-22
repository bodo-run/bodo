use async_trait::async_trait;
use serde_json::from_str;
use std::any::Any;

use crate::{
    errors::BodoError,
    graph::{Graph, NodeKind},
    plugin::Plugin,
    Result,
};

pub struct ResolverPlugin;

#[async_trait]
impl Plugin for ResolverPlugin {
    fn name(&self) -> &'static str {
        "ResolverPlugin"
    }

    async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        let mut edges = Vec::new();

        for node in &graph.nodes {
            if let NodeKind::Task(_) = &node.kind {
                if let Some(pre_deps) = node.metadata.get("pre_deps") {
                    let deps: Vec<String> = from_str(pre_deps)
                        .map_err(|e| BodoError::PluginError(format!("Invalid pre_deps: {}", e)))?;

                    for dep in deps {
                        let target_id = graph.task_registry.get(&dep).ok_or_else(|| {
                            BodoError::PluginError(format!("Dependency {} not found", dep))
                        })?;
                        edges.push((*target_id, node.id));
                    }
                }

                if let Some(post_deps) = node.metadata.get("post_deps") {
                    let deps: Vec<String> = from_str(post_deps)
                        .map_err(|e| BodoError::PluginError(format!("Invalid post_deps: {}", e)))?;

                    for dep in deps {
                        let target_id = graph.task_registry.get(&dep).ok_or_else(|| {
                            BodoError::PluginError(format!("Dependency {} not found", dep))
                        })?;
                        edges.push((node.id, *target_id));
                    }
                }
            }
        }

        for (from, to) in edges {
            graph.add_edge(from, to)?;
        }

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
