use async_trait::async_trait;
use serde_json::Map;
use serde_json::Value;

use crate::{errors::Result, graph::Graph};

pub struct PluginConfig {
    pub options: Option<Map<String, Value>>,
}

#[async_trait]
pub trait Plugin {
    fn name(&self) -> &'static str;
    async fn on_init(&mut self, config: &PluginConfig) -> Result<()>;
    async fn on_graph_build(&mut self, graph: &mut Graph) -> Result<()>;
}
