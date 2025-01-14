use async_trait::async_trait;

use crate::{
    errors::PluginError,
    graph::Graph,
    plugin::{Plugin, PluginConfig},
};

/// Handles concurrency (parallel tasks) and fail-fast logic.
pub struct ConcurrencyPlugin {
    pub concurrency_limit: usize,
    pub fail_fast: bool,
}

impl ConcurrencyPlugin {
    pub fn new(concurrency_limit: usize, fail_fast: bool) -> Self {
        ConcurrencyPlugin {
            concurrency_limit,
            fail_fast,
        }
    }
}

#[async_trait]
impl Plugin for ConcurrencyPlugin {
    fn name(&self) -> &'static str {
        "ConcurrencyPlugin"
    }

    async fn on_init(&mut self, config: &PluginConfig) -> Result<(), PluginError> {
        if let Some(options) = &config.options {
            if let Some(limit) = options.get("concurrency_limit") {
                if let Some(limit) = limit.as_u64() {
                    self.concurrency_limit = limit as usize;
                }
            }
            if let Some(fail_fast) = options.get("fail_fast") {
                if let Some(fail_fast) = fail_fast.as_bool() {
                    self.fail_fast = fail_fast;
                }
            }
        }
        Ok(())
    }

    async fn on_before_execute(&mut self, graph: &mut Graph) -> Result<(), PluginError> {
        // Store concurrency settings in metadata for the executor
        for node in &mut graph.nodes {
            node.metadata.insert(
                "concurrency.limit".to_string(),
                self.concurrency_limit.to_string(),
            );
            node.metadata.insert(
                "concurrency.fail_fast".to_string(),
                self.fail_fast.to_string(),
            );
        }
        Ok(())
    }
}
