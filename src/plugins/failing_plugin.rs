use async_trait::async_trait;
use std::any::Any;

use crate::{
    errors::BodoError,
    graph::Graph,
    plugin::{Plugin, PluginConfig},
    Result,
};

pub struct FakeFailingPlugin;

#[async_trait]
impl Plugin for FakeFailingPlugin {
    fn name(&self) -> &'static str {
        "FakeFailingPlugin"
    }

    async fn on_init(&mut self, _config: &PluginConfig) -> Result<()> {
        Err(BodoError::PluginError("Simulated init failure".to_string()))
    }

    async fn on_graph_build(&mut self, _graph: &mut Graph) -> Result<()> {
        Err(BodoError::PluginError(
            "Simulated build failure".to_string(),
        ))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
