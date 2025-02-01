// Re-export the unified Plugin interface and PluginManager
pub use crate::plugin::{Plugin, PluginConfig, PluginManager};

// Optionally keep module definitions for individual plugins:
pub mod concurrent_plugin;
pub mod env_plugin;
pub mod execution_plugin;
pub mod failing_plugin;
pub mod path_plugin;
pub mod prefix_plugin;
pub mod print_list_plugin;
pub mod timeout_plugin;
pub mod watch_plugin;
