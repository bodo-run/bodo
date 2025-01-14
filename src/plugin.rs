use async_trait::async_trait;
use serde_json::Map;
use serde_json::Value;
use std::any::Any;

use crate::{errors::Result, graph::Graph};

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

    pub fn get_watch_run_count(&self) -> u32 {
        if let Some(plugin) = self.plugins.iter().find(|p| p.name() == "watch") {
            if let Some(watch_plugin) = plugin
                .as_any()
                .downcast_ref::<crate::plugins::watch_plugin::WatchPlugin>()
            {
                return watch_plugin.get_run_count();
            }
        }
        0
    }
}
