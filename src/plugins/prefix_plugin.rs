use async_trait::async_trait;
use std::any::Any;

use crate::{
    graph::{Graph, NodeKind},
    plugin::{Plugin, PluginConfig},
    Result,
};

pub struct PrefixPlugin;

impl PrefixPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PrefixPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for PrefixPlugin {
    fn name(&self) -> &'static str {
        "PrefixPlugin"
    }

    fn priority(&self) -> i32 {
        50
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    async fn on_init(&mut self, config: &PluginConfig) -> Result<()> {
        if let Some(options) = &config.options {
            if let Some(prefix) = options.get("prefix") {
                if let Some(s) = prefix.as_str() {
                    // TODO: Implement prefix handling
                    let _ = s;
                }
            }
        }
        Ok(())
    }

    async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        for node in &graph.nodes {
            if let NodeKind::Task(task_data) = &node.kind {
                // TODO: Implement prefix handling
                let _ = task_data;
            }
        }
        Ok(())
    }
}
