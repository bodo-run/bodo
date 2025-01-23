use crate::{errors::BodoError, graph::Graph, Result};
use serde_json::{Map, Value};
use std::any::Any;

pub mod env_plugin;
pub mod execution_plugin;
pub mod path_plugin;
pub mod print_list_plugin;
pub mod resolver_plugin;
pub mod timeout_plugin;
pub mod watch_plugin;

#[async_trait::async_trait]
pub trait Plugin: Send {
    fn name(&self) -> &'static str;
    fn priority(&self) -> i32;
    fn as_any(&self) -> &dyn Any;
    async fn run(&mut self, graph: &mut Graph) -> Result<()>;
}

#[derive(Default)]
pub struct PluginConfig {
    pub fail_fast: bool,
    pub watch: bool,
    pub list: bool,
    pub options: Option<Map<String, Value>>,
}

pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub async fn run_lifecycle(
        &mut self,
        graph: &mut Graph,
        config: Option<PluginConfig>,
    ) -> Result<()> {
        let config = config.unwrap_or_default();

        // Sort plugins by priority
        self.plugins
            .sort_by_key(|p| std::cmp::Reverse(p.priority()));

        // Run each plugin
        for plugin in &mut self.plugins {
            plugin.run(graph).await?;
        }

        // Check for cycles after graph transformations
        if graph.has_cycle() {
            return Err(BodoError::PluginError(
                "Circular dependency detected in task graph".to_string(),
            ));
        }

        Ok(())
    }
}
