pub mod errors;
pub mod graph;
pub mod manager;
pub mod script_loader;

// Re-export commonly used items
pub use errors::PluginError;
pub use graph::{Graph, Node, NodeId, NodeKind};
pub use manager::GraphManager;
pub use script_loader::{BodoConfig, ScriptFile, TaskOrCommand};
