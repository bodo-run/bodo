pub mod errors;
pub mod graph;
pub mod manager;
pub mod plugin;
pub mod plugins;
pub mod script_loader;

pub use crate::errors::PluginError;
pub use crate::graph::{Graph, Node, NodeId, NodeKind};
pub use crate::manager::GraphManager;
pub use crate::plugin::{Plugin, PluginConfig};
pub use crate::script_loader::{BodoConfig, ScriptFile, TaskOrCommand};

// Re-export commonly used plugins
pub use crate::plugins::{
    command_exec_plugin::CommandExecPlugin, concurrency_plugin::ConcurrencyPlugin,
    env_var_plugin::EnvVarPlugin, list_plugin::ListPlugin, watch_plugin::WatchPlugin,
};
