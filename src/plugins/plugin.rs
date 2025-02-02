use crate::{errors::Result, graph::NodeKind};

pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn configure(&mut self, config: &PluginConfig) -> Result<()>;
    fn preprocess(&self, graph: &mut crate::graph::Graph) -> Result<()>;
    fn postprocess(&self, graph: &crate::graph::Graph) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct PluginConfig {
    // Configuration fields here
}

#[derive(Default)]
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn add_plugin(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }
}