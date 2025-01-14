use crate::{
    errors::PluginError,
    graph::Graph,
    plugin::{Plugin, PluginConfig},
};

/// The core manager orchestrates reading script configs, building the graph,
/// and coordinating plugin lifecycle calls.
pub struct GraphManager {
    pub graph: Graph,
    pub plugins: Vec<Box<dyn Plugin>>,
}

impl GraphManager {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            plugins: Vec::new(),
        }
    }

    /// Add a plugin to this manager's pipeline.
    pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    /// Initialize all plugins with their configs.
    pub fn init_plugins(&mut self, configs: &[PluginConfig]) -> Result<(), PluginError> {
        for plugin in self.plugins.iter_mut() {
            // Match plugin name to config, if found
            let config = configs
                .iter()
                .find(|c| c.plugin_name == plugin.name())
                .cloned()
                .unwrap_or_else(|| PluginConfig {
                    plugin_name: plugin.name().to_string(),
                    options: None,
                });

            plugin.on_init(&config)?;
        }
        Ok(())
    }

    /// Build the graph from script files or other sources.
    /// After building the graph, inform all plugins so they can modify or validate it.
    pub fn build_graph(&mut self) -> Result<(), PluginError> {
        // Let each plugin transform the graph as needed
        for plugin in self.plugins.iter_mut() {
            plugin.on_graph_build(&mut self.graph)?;
        }
        Ok(())
    }

    /// Execute tasks in the graph. Could be delegated to a plugin or a default executor.
    /// This is where concurrency, fail-fast, environment setup, etc. come together.
    pub fn execute(&mut self) -> Result<(), PluginError> {
        // Let plugins do a final transformation
        for plugin in self.plugins.iter_mut() {
            plugin.on_before_execute(&mut self.graph)?;
        }

        // Here is where the actual execution logic would go
        // ...
        // end of execution

        for plugin in self.plugins.iter_mut() {
            plugin.on_after_execute(&self.graph)?;
        }
        Ok(())
    }

    /// Utility for debugging the final graph.
    pub fn debug_graph(&self) {
        self.graph.print_debug();
    }
}

impl Default for GraphManager {
    fn default() -> Self {
        Self::new()
    }
}
