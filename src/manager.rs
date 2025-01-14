use crate::{
    errors::Result,
    graph::Graph,
    script_loader::{load_bodo_config, load_scripts, BodoConfig},
};
use std::path::PathBuf;

#[derive(Default)]
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

    pub async fn load_bodo_config(&mut self) -> Result<BodoConfig> {
        self.config = load_bodo_config(None)?;
        Ok(self.config.clone())
    }

    pub async fn build_graph(&mut self, paths: &[PathBuf]) -> Result<()> {
        load_scripts(paths, &mut self.graph)?;
        Ok(())
    }

    pub fn get_graph(&self) -> &Graph {
        &self.graph
    }
}
