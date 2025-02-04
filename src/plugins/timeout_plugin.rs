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

    pub fn parse_timeout(s: &str) -> Result<u64> {
        let duration = parse_duration(s).map_err(|e| {
            BodoError::PluginError(format!("Invalid timeout duration '{}': {}", s, e))
        })?;
        Ok(duration.as_secs())
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
                    let seconds = Self::parse_timeout(timeout_str)?;
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
}
