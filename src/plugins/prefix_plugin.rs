use async_trait::async_trait;
use std::any::Any;

use crate::{
    errors::Result,
    graph::{Graph, NodeKind},
    plugin::{Plugin, PluginConfig},
};

/// A plugin that sets a prefix for each node's output, e.g. "[build]" or "[taskX]".
pub struct PrefixPlugin {
    pub prefix_format: String,
}

impl PrefixPlugin {
    pub fn new() -> Self {
        PrefixPlugin {
            prefix_format: "[{}]".to_string(),
        }
    }
}

#[async_trait]
impl Plugin for PrefixPlugin {
    fn name(&self) -> &'static str {
        "PrefixPlugin"
    }

    async fn on_init(&mut self, config: &PluginConfig) -> Result<()> {
        if let Some(options) = &config.options {
            if let Some(format) = options.get("prefix_format") {
                if let Some(format) = format.as_str() {
                    self.prefix_format = format.to_string();
                }
            }
        }
        Ok(())
    }

    async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        for node in &mut graph.nodes {
            let prefix = match &node.kind {
                NodeKind::Task(td) => self.prefix_format.replace("{}", &td.name),
                NodeKind::Command(cd) => {
                    let short_cmd = cd
                        .raw_command
                        .split_whitespace()
                        .next()
                        .unwrap_or("cmd")
                        .to_string();
                    self.prefix_format.replace("{}", &short_cmd)
                }
            };
            node.metadata.insert("prefix".to_string(), prefix);
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
