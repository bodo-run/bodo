pub mod errors;
pub mod graph;
pub mod manager;
pub mod plugin;
pub mod plugins;
pub mod script_loader;

pub use crate::errors::{PluginError, Result};
pub use crate::graph::{CommandData, Edge, EdgeType, Graph, Node, NodeId, NodeKind, TaskData};
pub use crate::manager::GraphManager;
pub use crate::plugin::{Plugin, PluginConfig};
pub use crate::script_loader::{BodoConfig, ScriptFile, TaskOrCommand};

// Re-export commonly used plugins
pub use crate::plugins::{
    command_exec_plugin::CommandExecPlugin, concurrency_plugin::ConcurrencyPlugin,
    env_var_plugin::EnvVarPlugin, list_plugin::ListPlugin, path_plugin::PathPlugin,
    prefix_plugin::PrefixPlugin, watch_plugin::WatchPlugin,
};
