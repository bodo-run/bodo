use crate::errors::PluginError;
use crate::graph::Graph;
use crate::plugin::{Plugin, PluginConfig};

/// Implements watch mode: monitors file changes and re-runs tasks.
pub struct WatchPlugin;

impl WatchPlugin {
    pub fn new() -> Self {
        WatchPlugin
    }
}

impl Plugin for WatchPlugin {
    fn name(&self) -> &'static str {
        "WatchPlugin"
    }

    fn on_init(&mut self, _config: &PluginConfig) -> Result<(), PluginError> {
        // Possibly set up file watchers
        Ok(())
    }

    fn on_before_execute(&mut self, _graph: &mut Graph) -> Result<(), PluginError> {
        // Potentially skip or postpone execution until file changes
        Ok(())
    }
}

impl Default for WatchPlugin {
    fn default() -> Self {
        Self::new()
    }
}
