use crate::errors::PluginError;
use crate::graph::Graph;

/// Configuration passed to a plugin at initialization time.
#[derive(Debug, Clone)]
pub struct PluginConfig {
    // Could hold plugin-specific config data loaded from YAML or other files
    pub plugin_name: String,
    pub options: Option<serde_json::Value>,
}

/// Plugin lifecycle trait. Each plugin can tap into relevant phases.
pub trait Plugin {
    fn name(&self) -> &'static str;

    /// Called once at startup for plugin-specific config or initialization.
    fn on_init(&mut self, _config: &PluginConfig) -> Result<(), PluginError> {
        Ok(())
    }

    /// Called after Graph is built but before execution. Allows plugins to modify the graph.
    fn on_graph_build(&mut self, _graph: &mut Graph) -> Result<(), PluginError> {
        Ok(())
    }

    /// Called right before tasks are executed. Good for final transformations or validations.
    fn on_before_execute(&mut self, _graph: &mut Graph) -> Result<(), PluginError> {
        Ok(())
    }

    /// Called after tasks complete or fail. Could be used for cleanup or logging.
    fn on_after_execute(&mut self, _graph: &Graph) -> Result<(), PluginError> {
        Ok(())
    }
}
