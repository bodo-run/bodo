use crate::errors::PluginError;
use crate::graph::Graph;
use crate::plugin::{Plugin, PluginConfig};

/// Actually runs commands using Tokio or other async runtime.
pub struct CommandExecPlugin;

impl CommandExecPlugin {
    pub fn new() -> Self {
        CommandExecPlugin
    }
}

impl Plugin for CommandExecPlugin {
    fn name(&self) -> &'static str {
        "CommandExecPlugin"
    }

    fn on_init(&mut self, _config: &PluginConfig) -> Result<(), PluginError> {
        Ok(())
    }

    fn on_before_execute(&mut self, _graph: &mut Graph) -> Result<(), PluginError> {
        // Prepare command execution environment, prefixes, etc.
        Ok(())
    }

    // Optionally override execution with a custom method, if we design the trait that way
}

impl Default for CommandExecPlugin {
    fn default() -> Self {
        Self::new()
    }
}
