use crate::{
    errors::PluginError,
    graph::Graph,
    script_loader::{self, BodoConfig},
};

/// The manager orchestrates reading script configs, building the graph.
pub struct GraphManager {
    pub graph: Graph,
    pub config: BodoConfig,
}

impl GraphManager {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            config: BodoConfig::default(),
        }
    }

    /// Load bodo.toml if present, or keep defaults.
    pub fn load_bodo_config(&mut self, config_path: Option<&str>) -> Result<(), PluginError> {
        self.config = script_loader::load_bodo_config(config_path)?;
        Ok(())
    }

    /// Build the graph from files in the filesystem.
    /// This is effectively "script_loader::load_scripts_from_fs" plus any post-processing.
    pub fn build_graph(&mut self) -> Result<(), PluginError> {
        script_loader::load_scripts_from_fs(&self.config, &mut self.graph)?;
        // Potentially do validations or detect cycles, etc.
        Ok(())
    }

    pub fn debug_graph(&self) {
        self.graph.print_debug();
    }
}

impl Default for GraphManager {
    fn default() -> Self {
        Self::new()
    }
}
