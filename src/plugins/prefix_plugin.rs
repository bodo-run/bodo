use async_trait::async_trait;
use std::any::Any;

use crate::{
    graph::{Graph, NodeKind},
    plugin::{Plugin, PluginConfig},
    Result,
};

pub struct PrefixPlugin {
    prefix: Option<String>,
}

impl PrefixPlugin {
    pub fn new() -> Self {
        Self { prefix: None }
    }
}

#[async_trait]
impl Plugin for PrefixPlugin {
    fn name(&self) -> &'static str {
        "PrefixPlugin"
    }

    async fn on_init(&mut self, config: &PluginConfig) -> Result<()> {
        if let Some(options) = &config.options {
            if let Some(prefix) = options.get("prefix") {
                if let Some(s) = prefix.as_str() {
                    self.prefix = Some(s.to_string());
                }
            }
        }
        Ok(())
    }

    async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        if let Some(ref prefix) = self.prefix {
            for node in &mut graph.nodes {
                match &mut node.kind {
                    NodeKind::Task(task_data) => {
                        if let Some(desc) = &task_data.description {
                            task_data.description = Some(format!("{}{}", prefix, desc));
                        }
                    }
                    NodeKind::Command(cmd_data) => {
                        if let Some(desc) = &cmd_data.description {
                            cmd_data.description = Some(format!("{}{}", prefix, desc));
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
