pub mod errors;
pub mod graph;
pub mod manager;
pub mod plugin;
pub mod plugins;
pub mod script_loader;

pub use errors::BodoError;
pub use errors::Result;
pub use graph::{CommandData, Graph, NodeKind, TaskData};
pub use manager::GraphManager;
pub use plugin::{Plugin, PluginConfig, PluginManager};
pub use script_loader::{load_bodo_config, load_scripts, BodoConfig};
