use serde_json::{Map, Value};
use std::any::Any;

use crate::errors::BodoError;
use crate::graph::{Graph, NodeId};
use crate::Result;

#[async_trait::async_trait]
pub trait Plugin: Send {
    fn name(&self) -> &'static str;
    fn priority(&self) -> i32;
    fn as_any(&self) -> &dyn Any;
    async fn on_init(&mut self, config: &PluginConfig) -> Result<()> {
        let _ = config;
        Ok(())
    }
    async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        let _ = graph;
        Ok(())
    }
    async fn on_after_run(&mut self, graph: &mut Graph) -> Result<()> {
        let _ = graph;
        Ok(())
    }
    async fn on_run(&mut self, node_id: NodeId, graph: &mut Graph) -> Result<()> {
        let _ = (node_id, graph);
        Ok(())
    }
}

#[derive(Default)]
pub struct PluginConfig {
    pub fail_fast: bool,
    pub watch: bool,
    pub list: bool,
    pub options: Option<Map<String, Value>>,
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

    pub fn plugin_names(&self) -> Vec<String> {
        self.plugins.iter().map(|p| p.name().to_string()).collect()
    }

    pub async fn run_lifecycle(
        &mut self,
        graph: &mut Graph,
        config: Option<PluginConfig>,
    ) -> Result<()> {
        let config = config.unwrap_or_default();

        // Sort plugins by priority
        self.plugins
            .sort_by_key(|p| std::cmp::Reverse(p.priority()));

        // Phase 1: on_init
        for plugin in &mut self.plugins {
            plugin.on_init(&config).await?;
        }

        // Phase 2: on_graph_build
        for plugin in &mut self.plugins {
            plugin.on_graph_build(graph).await?;
        }

        // Check for cycles after graph transformations
        if graph.has_cycle() {
            return Err(BodoError::PluginError(
                "Circular dependency detected in task graph".to_string(),
            ));
        }

        // Phase 3: on_after_run
        for plugin in &mut self.plugins {
            plugin.on_after_run(graph).await?;
        }

        Ok(())
    }

    pub async fn on_run_node(&mut self, node_id: NodeId, graph: &mut Graph) -> Result<()> {
        for plugin in self.plugins.iter_mut() {
            plugin.on_run(node_id, graph).await?;
        }
        Ok(())
    }

    pub fn sort_plugins(&mut self) {
        self.plugins
            .sort_by_key(|b| std::cmp::Reverse(b.priority()));
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
