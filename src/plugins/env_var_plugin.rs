use crate::errors::PluginError;
use crate::graph::Graph;
use crate::plugin::{Plugin, PluginConfig};

/// Manages environment variables for tasks and commands.
pub struct EnvVarPlugin {
    // Could store global or per-task environment settings
}

impl EnvVarPlugin {
    pub fn new() -> Self {
        EnvVarPlugin {}
    }
}

impl Plugin for EnvVarPlugin {
    fn name(&self) -> &'static str {
        "EnvVarPlugin"
    }

    fn on_init(&mut self, _config: &PluginConfig) -> Result<(), PluginError> {
        // load env var config, if any
        Ok(())
    }

    fn on_graph_build(&mut self, _graph: &mut Graph) -> Result<(), PluginError> {
        // attach environment info to each node, handle merges, etc.
        Ok(())
    }
}

impl Default for EnvVarPlugin {
    fn default() -> Self {
        Self::new()
    }
}
