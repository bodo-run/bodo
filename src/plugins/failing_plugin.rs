use async_trait::async_trait;
use std::any::Any;

use crate::{
    errors::{BodoError, Result},
    graph::Graph,
    plugin::{Plugin, PluginConfig},
};

/// A plugin that deliberately fails during `on_init` or `on_graph_build`.
/// Useful for testing error handling in the PluginManager.
#[derive(Default)]
pub struct FakeFailingPlugin {
    pub fail_on_init: bool,
    pub fail_on_graph_build: bool,
}

impl FakeFailingPlugin {
    pub fn new(fail_on_init: bool, fail_on_graph_build: bool) -> Self {
        Self {
            fail_on_init,
            fail_on_graph_build,
        }
    }
}

#[async_trait]
impl Plugin for FakeFailingPlugin {
    fn name(&self) -> &'static str {
        "FakeFailingPlugin"
    }

    async fn on_init(&mut self, _config: &PluginConfig) -> Result<()> {
        if self.fail_on_init {
            return Err(BodoError::PluginError(
                "Intentional failure in on_init".to_string(),
            ));
        }
        Ok(())
    }

    async fn on_graph_build(&mut self, _graph: &mut Graph) -> Result<()> {
        if self.fail_on_graph_build {
            return Err(BodoError::PluginError(
                "Intentional failure in on_graph_build".to_string(),
            ));
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn on_task_start(&mut self) {
        // Nothing to do on task start for this plugin
    }
}
