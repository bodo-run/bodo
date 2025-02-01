use async_trait::async_trait;
use std::any::Any;

use crate::{
    errors::{BodoError, Result},
    graph::{Graph, NodeKind},
    plugin::Plugin,
};
use humantime::parse_duration;

pub struct TimeoutPlugin;

impl Default for TimeoutPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeoutPlugin {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Plugin for TimeoutPlugin {
    fn name(&self) -> &'static str {
        "TimeoutPlugin"
    }

    fn priority(&self) -> i32 {
        75 // Positioned between concurrency and watch plugins
    }

    async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        for node in &mut graph.nodes {
            if let NodeKind::Task(_) = &node.kind {
                if let Some(timeout_str) = node.metadata.get("timeout") {
                    let seconds = parse_timeout(timeout_str)?;
                    node.metadata
                        .insert("timeout_seconds".to_string(), seconds.to_string());
                }
            }
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn parse_timeout(s: &str) -> Result<u64> {
    let duration = parse_duration(s)
        .map_err(|e| BodoError::PluginError(format!("Invalid timeout duration '{}': {}", s, e)))?;
    Ok(duration.as_secs())
}
