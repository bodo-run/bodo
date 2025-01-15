use async_trait::async_trait;
use serde_json::Map;
use serde_json::Value;
use std::any::Any;

use crate::{errors::Result, graph::Graph};

/// Represents the major phases in which the plugin manager invokes plugins.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginExecutionPhase {
    /// Manager is about to call `on_init` on the plugin.
    InitStart,
    /// Manager just finished calling `on_init` on the plugin.
    InitEnd,
    /// Manager is about to call `on_graph_build`.
    GraphBuildStart,
    /// Manager just finished calling `on_graph_build`.
    GraphBuildEnd,
    /// Manager is about to invoke `on_task_start`.
    TaskStartBegin,
    /// Manager just finished `on_task_start`.
    TaskStartEnd,
}

/// Provides context for the plugin about the plugin execution order.
#[derive(Debug)]
pub struct PluginExecutionContext<'a> {
    /// All plugin names in the order they are registered.
    pub all_plugin_names: &'a [String],
    /// Index of the current plugin in `all_plugin_names`.
    pub current_plugin_index: usize,
    /// The phase that is about to happen, or just happened.
    pub phase: PluginExecutionPhase,
}

#[derive(Default)]
pub struct PluginConfig {
    pub options: Option<Map<String, Value>>,
}

#[async_trait]
pub trait Plugin: Send + Any {
    fn name(&self) -> &'static str;

    /// Called by the manager to let the plugin know about each major lifecycle event.
    /// (Optional to implement; default is a no-op.)
    async fn on_lifecycle_event(&mut self, _ctx: &PluginExecutionContext<'_>) -> Result<()> {
        Ok(())
    }

    async fn on_init(&mut self, config: &PluginConfig) -> Result<()>;
    async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()>;
    fn on_task_start(&mut self);
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

    /// Return the plugin names in the order they were registered
    fn plugin_names(&self) -> Vec<String> {
        self.plugins.iter().map(|p| p.name().to_string()).collect()
    }

    pub async fn init_plugins(&mut self, config: &PluginConfig) -> Result<()> {
        let names = self.plugin_names();
        for (index, plugin) in self.plugins.iter_mut().enumerate() {
            let ctx_start = PluginExecutionContext {
                all_plugin_names: &names,
                current_plugin_index: index,
                phase: PluginExecutionPhase::InitStart,
            };
            plugin.on_lifecycle_event(&ctx_start).await?;

            plugin.on_init(config).await?;

            let ctx_end = PluginExecutionContext {
                all_plugin_names: &names,
                current_plugin_index: index,
                phase: PluginExecutionPhase::InitEnd,
            };
            plugin.on_lifecycle_event(&ctx_end).await?;
        }
        Ok(())
    }

    pub async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()> {
        let names = self.plugin_names();
        for (index, plugin) in self.plugins.iter_mut().enumerate() {
            let ctx_start = PluginExecutionContext {
                all_plugin_names: &names,
                current_plugin_index: index,
                phase: PluginExecutionPhase::GraphBuildStart,
            };
            plugin.on_lifecycle_event(&ctx_start).await?;

            plugin.on_graph_build(graph).await?;

            let ctx_end = PluginExecutionContext {
                all_plugin_names: &names,
                current_plugin_index: index,
                phase: PluginExecutionPhase::GraphBuildEnd,
            };
            plugin.on_lifecycle_event(&ctx_end).await?;
        }
        Ok(())
    }

    pub fn on_task_start(&mut self) {
        let names = self.plugin_names();
        for (index, plugin) in self.plugins.iter_mut().enumerate() {
            // Before
            let ctx_start = PluginExecutionContext {
                all_plugin_names: &names,
                current_plugin_index: index,
                phase: PluginExecutionPhase::TaskStartBegin,
            };
            let _ = futures::executor::block_on(plugin.on_lifecycle_event(&ctx_start));

            plugin.on_task_start();

            // After
            let ctx_end = PluginExecutionContext {
                all_plugin_names: &names,
                current_plugin_index: index,
                phase: PluginExecutionPhase::TaskStartEnd,
            };
            let _ = futures::executor::block_on(plugin.on_lifecycle_event(&ctx_end));
        }
    }
}
