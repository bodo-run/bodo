pub mod cli;
pub mod config;
pub mod designer;
pub mod errors;
pub mod graph;
pub mod manager;
pub mod plugin;
pub mod plugins;
pub mod process;
pub mod recovery;
pub mod script_loader; // Added empty designer module for coverage

pub use config::BodoConfig;
pub use errors::{BodoError, Result};
pub use graph::Graph;
pub use manager::GraphManager;
pub use plugin::{Plugin, PluginConfig, PluginManager};
