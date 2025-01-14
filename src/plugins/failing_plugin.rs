use crate::{
    errors::{BodoError, Result},
    graph::Graph,
    plugin::{Plugin, PluginConfig},
};
use async_trait::async_trait;
use std::any::Any;

/// A plugin that fails on init or on graph build, depending on flags.
pub struct FailingPlugin {
    fail_on_init: bool,
    fail_on_graph_build: bool,
}

impl FailingPlugin {
    pub fn new(fail_on_init: bool, fail_on_graph_build: bool) -> Self {
        Self {
            fail_on_init,
            fail_on_graph_build,
        }
    }
}

#[async_trait]
impl Plugin for FailingPlugin {
    fn name(&self) -> &'static str {
        "FailingPlugin"
    }

    async fn on_init(&mut self, _config: &PluginConfig) -> Result<()> {
        if self.fail_on_init {
            Err(BodoError::PluginError(
                "FailingPlugin forced fail on init".into(),
            ))
        } else {
            Ok(())
        }
    }

    async fn on_graph_build(&mut self, _graph: &mut Graph) -> Result<()> {
        if self.fail_on_graph_build {
            Err(BodoError::PluginError(
                "FailingPlugin forced fail on graph build".into(),
            ))
        } else {
            Ok(())
        }
    }

    fn on_task_start(&mut self) {}

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// A plugin that always succeeds. Useful for verifying others still run if we ignore failures.
pub struct SucceedingPlugin;

impl SucceedingPlugin {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Plugin for SucceedingPlugin {
    fn name(&self) -> &'static str {
        "SucceedingPlugin"
    }

    async fn on_init(&mut self, _config: &PluginConfig) -> Result<()> {
        Ok(())
    }

    async fn on_graph_build(&mut self, _graph: &mut Graph) -> Result<()> {
        Ok(())
    }

    fn on_task_start(&mut self) {}

    fn as_any(&self) -> &dyn Any {
        self
    }
}
