use crate::graph::Graph;
use crate::Result;
use std::any::Any;

/// Synchronous plugin trait (no `async` anymore).
pub trait Plugin: Send {
    fn name(&self) -> &'static str;
    fn priority(&self) -> i32;
    fn as_any(&self) -> &dyn Any;

    /// Called after plugin is created, before building the graph.
    fn on_init(&mut self, _config: &PluginConfig) -> Result<()> {
        Ok(())
    }

    /// Called when building/modifying the graph (e.g. adding concurrency).
    fn on_graph_build(&mut self, _graph: &mut Graph) -> Result<()> {
        Ok(())
    }

    /// Called after the graph is built but before final execution.
    fn on_after_run(&mut self, _graph: &mut Graph) -> Result<()> {
        Ok(())
    }

    /// Called each time we run an individual node (not used here by default).
    fn on_run(&mut self, _node_id: usize, _graph: &mut Graph) -> Result<()> {
        Ok(())
    }
}

#[derive(Default)]
pub struct PluginConfig {
    pub fail_fast: bool,
    pub watch: bool,
    pub list: bool,
    pub dry_run: bool, // Added dry_run to PluginConfig
    pub options: Option<serde_json::Map<String, serde_json::Value>>,
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

    pub fn sort_plugins(&mut self) {
        self.plugins
            .sort_by_key(|p| std::cmp::Reverse(p.priority()));
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    /// Provide read-only access to the plugins, for testing purposes
    pub fn get_plugins(&self) -> &[Box<dyn Plugin>] {
        &self.plugins
    }

    /// Runs the "lifecycle" in a blocking (synchronous) manner.
    pub fn run_lifecycle(&mut self, graph: &mut Graph, config: Option<PluginConfig>) -> Result<()> {
        let config = config.unwrap_or_default();
        self.sort_plugins();

        // on_init
        for plugin in &mut self.plugins {
            plugin.on_init(&config)?;
        }
        // on_graph_build
        for plugin in &mut self.plugins {
            plugin.on_graph_build(graph)?;
        }
        // on_after_run
        for plugin in &mut self.plugins {
            plugin.on_after_run(graph)?;
        }
        Ok(())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
