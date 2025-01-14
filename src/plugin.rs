use async_trait::async_trait;
use serde_json::Map;
use serde_json::Value;
use std::any::Any;

use crate::{errors::Result, graph::Graph};

#[derive(Default)]
pub struct PluginConfig {
    pub options: Option<Map<String, Value>>,
}

#[async_trait]
pub trait Plugin: Send + Any {
    fn name(&self) -> &'static str;
    async fn on_init(&mut self, config: &PluginConfig) -> Result<()>;
    async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()>;
    fn on_task_start(&mut self) {}
    fn as_any(&self) -> &dyn Any;
}

pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub async fn init_plugins(&mut self, config: &PluginConfig) -> Result<()> {
        for plugin in &mut self.plugins {
            plugin.on_init(config).await?;
        }
        Ok(())
    }

    pub async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        for plugin in &mut self.plugins {
            plugin.on_graph_build(graph).await?;
        }
        Ok(())
    }
}
