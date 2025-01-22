use async_trait::async_trait;
use serde_json::{Map, Value};
use std::any::Any;

use crate::errors::BodoError;
use crate::graph::{Graph, NodeId};
use crate::Result;

#[derive(Default)]
pub struct PluginConfig {
    pub options: Option<Map<String, Value>>,
}

#[async_trait]
pub trait Plugin: Send + Any + Sync {
    /// Returns the execution priority for this plugin. Plugins with higher
    /// priority values execute earlier in the lifecycle phases. The default
    /// priority is 0.
    fn priority(&self) -> i32 {
        0
    }

    fn name(&self) -> &'static str;

    /// Called first to load or parse plugin config
    async fn on_init(&mut self, config: &PluginConfig) -> Result<()> {
        let _ = config;
        Ok(())
    }

    /// Called after the scripts or config have been loaded into a Graph, but before plugins mutate it
    async fn on_before_graph_build(&mut self, _graph: &mut Graph) -> Result<()> {
        Ok(())
    }

    /// Called to transform or annotate the Graph (resolving tasks, concurrency, watchers, etc.)
    async fn on_graph_build(&mut self, _graph: &mut Graph) -> Result<()> {
        Ok(())
    }

    /// Called just before execution starts, after the graph transformations are done
    async fn on_before_run(&mut self, _graph: &mut Graph) -> Result<()> {
        Ok(())
    }

    /// Called for each node that is about to run (or as it runs)
    async fn on_run(&mut self, _node_id: NodeId, _graph: &mut Graph) -> Result<()> {
        Ok(())
    }

    /// Called after all tasks have completed
    async fn on_after_run(&mut self, _graph: &mut Graph) -> Result<()> {
        Ok(())
    }

    fn as_any(&self) -> &dyn Any;
}

#[derive(Default)]
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

    pub async fn run_lifecycle(&mut self, graph: &mut Graph, cfg: &PluginConfig) -> Result<()> {
        self.plugins.sort_by(|a, b| b.priority().cmp(&a.priority()));
        // Phase 1: on_init
        for plugin in self.plugins.iter_mut() {
            plugin.on_init(cfg).await?;
        }

        // Phase 2: on_before_graph_build
        for plugin in self.plugins.iter_mut() {
            plugin.on_before_graph_build(graph).await?;
        }

        // Phase 3: on_graph_build
        for plugin in self.plugins.iter_mut() {
            plugin.on_graph_build(graph).await?;
        }

        // Check for cycles after graph transformations
        if graph.has_cycle() {
            return Err(BodoError::PluginError(
                "Cycle detected in graph".to_string(),
            ));
        }

        // Phase 4: on_before_run
        for plugin in self.plugins.iter_mut() {
            plugin.on_before_run(graph).await?;
        }

        // Phase 5: on_run is called by the execution plugin for each node

        // Phase 6: on_after_run
        for plugin in self.plugins.iter_mut() {
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
}
