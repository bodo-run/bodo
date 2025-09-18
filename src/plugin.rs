use crate::graph::Graph;
use crate::Result;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;

/// Synchronous plugin trait (no `async` anymore).
pub trait Plugin: Send {
    fn name(&self) -> &'static str;
    fn priority(&self) -> i32;
    fn as_any(&self) -> &dyn Any;

    /// Called after plugin is created, before building the graph.
    fn on_init(&mut self, _config: &PluginConfig) -> Result<()> {
        Ok(())
    }

    /// Called when building/modifying the graph (e.g. adding concurrency).
    fn on_graph_build(&mut self, _graph: &mut Graph) -> Result<()> {
        Ok(())
    }

    /// Called after the graph is built but before final execution.
    fn on_after_run(&mut self, _graph: &mut Graph) -> Result<()> {
        Ok(())
    }

    /// Called each time we run an individual node (not used here by default).
    fn on_run(&mut self, _node_id: usize, _graph: &mut Graph) -> Result<()> {
        Ok(())
    }
}

#[derive(Default)]
pub struct PluginConfig {
    pub fail_fast: bool,
    pub watch: bool,
    pub list: bool,
    pub options: Option<serde_json::Map<String, serde_json::Value>>,
}

/// Represents a single simulated action (e.g., a command execution).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatedAction {
    /// Type of action (e.g., "command", "file_write").
    pub action_type: String,
    /// Description of the action.
    pub description: String,
    /// Details specific to the action (e.g., expanded command string).
    pub details: HashMap<String, String>,
    /// Node ID in the graph this action relates to.
    pub node_id: Option<usize>,
}

/// Report structure for dry-run simulations.
/// Contains details of what would be executed without side effects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DryRunReport {
    /// Plugin that generated this report.
    pub plugin_name: String,
    /// List of simulated commands or actions.
    pub simulated_actions: Vec<SimulatedAction>,
    /// Dependencies that would be resolved (e.g., task names, file paths).
    pub dependencies: Vec<String>,
    /// Warnings or notes about the simulation (e.g., missing env vars).
    pub warnings: Vec<String>,
    /// Estimated execution time or other metadata (optional).
    pub metadata: HashMap<String, String>,
}

/// Aggregated report from all plugins.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedDryRunReport {
    pub reports: Vec<DryRunReport>,
}

/// Trait for plugins that support dry-run simulation.
/// Implementations should simulate their behavior without side effects.
pub trait DryRun: Send {
    /// Simulate the plugin's behavior for dry-run mode.
    /// Returns a report of what would be executed or modified.
    fn dry_run_simulate(&self, graph: &Graph, config: &PluginConfig) -> Result<DryRunReport>;
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

    pub fn sort_plugins(&mut self) {
        self.plugins
            .sort_by_key(|p| std::cmp::Reverse(p.priority()));
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    /// Provide read-only access to the plugins, for testing purposes
    pub fn get_plugins(&self) -> &[Box<dyn Plugin>] {
        &self.plugins
    }

    /// Provide mutable access to the plugins for internal operations
    pub fn get_plugins_mut(&mut self) -> &mut [Box<dyn Plugin>] {
        &mut self.plugins
    }

    /// Runs the "lifecycle" in a blocking (synchronous) manner.
    pub fn run_lifecycle(&mut self, graph: &mut Graph, config: Option<PluginConfig>) -> Result<()> {
        let config = config.unwrap_or_default();
        self.sort_plugins();

        // on_init
        for plugin in &mut self.plugins {
            plugin.on_init(&config)?;
        }
        // on_graph_build
        for plugin in &mut self.plugins {
            plugin.on_graph_build(graph)?;
        }
        // on_after_run
        for plugin in &mut self.plugins {
            plugin.on_after_run(graph)?;
        }
        Ok(())
    }

    /// Run dry-run simulation for all supporting plugins.
    pub fn dry_run(&self, graph: &Graph, config: &PluginConfig) -> Result<AggregatedDryRunReport> {
        let mut reports = Vec::new();
        for plugin in &self.plugins {
            // Use a more generic approach - check if plugin supports dry run
            // by attempting to cast to each known plugin type that implements DryRun
            if let Some(execution_plugin) = plugin
                .as_any()
                .downcast_ref::<crate::plugins::execution_plugin::ExecutionPlugin>(
            ) {
                reports.push(execution_plugin.dry_run_simulate(graph, config)?);
            }
            // Future plugins implementing DryRun can be added here
            // if let Some(other_plugin) = plugin.as_any().downcast_ref::<OtherPlugin>() {
            //     reports.push(other_plugin.dry_run_simulate(graph, config)?);
            // }
        }
        Ok(AggregatedDryRunReport { reports })
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
