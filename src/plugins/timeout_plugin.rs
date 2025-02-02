use humantime::parse_duration;
use log::debug;
use std::any::Any;

use crate::{
    errors::{BodoError, Result},
    graph::{Graph, NodeKind},
    plugin::Plugin,
};

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

impl Plugin for TimeoutPlugin {
    fn name(&self) -> &'static str {
        "TimeoutPlugin"
    }

    fn priority(&self) -> i32 {
        75
    }

    fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        for node in &mut graph.nodes {
            if let NodeKind::Task(_) = &node.kind {
                if let Some(timeout_str) = node.metadata.get("timeout") {
                    let seconds = parse_timeout(timeout_str)?;
                    node.metadata
                        .insert("timeout_seconds".to_string(), seconds.to_string());
                    debug!(
                        "TimeoutPlugin: node {} has timeout of {} seconds",
                        node.id, seconds
                    );
                }
            }
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

fn parse_timeout(timeout_str: &str) -> Result<u64> {
    parse_duration(timeout_str)
        .map(|d| d.as_secs())
        .map_err(|e| BodoError::PluginError(format!("Invalid timeout format: {}", e)))
}
