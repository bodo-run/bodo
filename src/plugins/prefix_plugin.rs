use async_trait::async_trait;
use std::any::Any;

use crate::{
    errors::Result,
    graph::{Graph, NodeKind},
    plugin::{Plugin, PluginConfig},
};

/// A plugin that sets a prefix for each node's output, e.g. "[build]" or "[taskX]".
#[derive(Default)]
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
                NodeKind::Task(task_data) => {
                    if let Some(script_name) = &task_data.script_name {
                        format!("[{}] ", script_name)
                    } else {
                        String::new()
                    }
                }
                NodeKind::Command(_) => String::new(),
                NodeKind::ScriptFile(script_data) => format!("[{}] ", script_data.name),
                NodeKind::RootScriptFile(_) => String::new(),
            };
            node.metadata.insert("prefix".to_string(), prefix);
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn on_task_start(&mut self) {
        // Nothing to do on task start for this plugin
    }
}
