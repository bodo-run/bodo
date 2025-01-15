use crate::{
    errors::{BodoError, Result},
    graph::{Graph, NodeKind},
    plugin::{Plugin, PluginConfig, PluginManager},
    script_loader::{load_bodo_config, load_scripts, BodoConfig},
};
use std::path::PathBuf;

/// The `GraphManager` is responsible for:
/// 1. Loading the Bodo config (bodo.yaml, bodo.json, etc.)
/// 2. Discovering script files via the config and building a `Graph` from them
/// 3. Applying transformations via the plugin system
/// 4. Exposing the final `Graph` so tasks can be executed
#[derive(Default)]
pub struct GraphManager {
    pub graph: Graph,
    pub config: BodoConfig,
    pub plugin_manager: PluginManager,
}

impl GraphManager {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            config: BodoConfig::default(),
            plugin_manager: PluginManager::new(),
        }
    }

    /// Register a plugin with the manager
    pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>) {
        self.plugin_manager.register(plugin);
    }

    /// 1) Loads the BodoConfig from a file path or the default fallback
    /// 2) Stores it internally for use by subsequent steps
    pub async fn load_bodo_config(&mut self, config_path: Option<&str>) -> Result<BodoConfig> {
        self.config = load_bodo_config(config_path)?;
        Ok(self.config.clone())
    }

    /// 1) Discovers script files using the fields in self.config
    /// 2) Calls `load_scripts` to parse tasks/commands into `self.graph`
    /// 3) Optionally returns an error if something went wrong
    pub async fn build_graph(&mut self) -> Result<()> {
        let mut paths_to_load = vec![];

        // Load from scripts_dir if configured
        if let Some(ref scripts_dir) = self.config.scripts_dir {
            let dir_path = PathBuf::from(scripts_dir);
            if dir_path.exists() && dir_path.is_dir() {
                if let Some(ref pattern) = self.config.scripts_glob {
                    // Use glob to find matching files
                    let glob_pattern = dir_path.join(pattern).to_string_lossy().into_owned();
                    for entry in glob::glob(&glob_pattern)
                        .map_err(|e| BodoError::PluginError(e.to_string()))?
                    {
                        match entry {
                            Ok(path) => paths_to_load.push(path),
                            Err(e) => return Err(BodoError::PluginError(e.to_string())),
                        }
                    }
                } else {
                    // Default to loading all .yaml files
                    paths_to_load.push(dir_path.join("*.yaml"));
                }
            }
        }

        // Actually parse tasks/commands from the discovered files
        if !paths_to_load.is_empty() {
            load_scripts(&paths_to_load, &mut self.graph)?;
        }

        Ok(())
    }

    /// 1) Initializes all registered plugins with the given options (if any)
    /// 2) If any plugin fails on_init, returns an error
    pub async fn init_plugins(&mut self, plugin_options: Option<PluginConfig>) -> Result<()> {
        let cfg = plugin_options.unwrap_or_default();
        self.plugin_manager.init_plugins(&cfg).await?;
        Ok(())
    }

    /// 1) Calls the plugins' `on_graph_build` hooks to transform the graph
    /// 2) If any plugin fails, returns an error
    pub async fn apply_plugins_to_graph(&mut self) -> Result<()> {
        self.plugin_manager.on_graph_build(&mut self.graph).await?;
        Ok(())
    }

    /// Retrieve the final `Graph` after scripts have been loaded and all plugins have run
    pub fn get_graph(&self) -> &Graph {
        &self.graph
    }

    /// Find a task in the graph by name and return its node index if found
    pub fn find_task(&self, task_name: &str) -> Option<usize> {
        self.graph
            .nodes
            .iter()
            .position(|n| matches!(&n.kind, NodeKind::Task(t) if t.name == task_name))
    }

    /// Example function to find a node in the graph by name
    /// and run it or do something with it (like scheduling execution).
    /// In a real system, you'd have concurrency, watchers, etc.
    pub async fn run_task(&mut self, task_name: &str) -> Result<()> {
        let node_idx = self.find_task(task_name).ok_or_else(|| {
            BodoError::PluginError(format!("Task '{}' not found in the graph", task_name))
        })?;

        let node = &self.graph.nodes[node_idx];
        if let NodeKind::Task(task_data) = &node.kind {
            if let Some(command) = &task_data.command {
                // Notify plugins that a task is starting
                self.plugin_manager.on_task_start();

                // In a real implementation, you'd:
                // 1. Check for circular dependencies
                // 2. Run pre_deps
                // 3. Execute the command (possibly via an executor plugin)
                // 4. Run post_deps
                // 5. Handle concurrency
                println!("Would run command: {}", command);
            } else {
                println!("No command specified for task '{}'", task_name);
            }
            Ok(())
        } else {
            Err(BodoError::PluginError(format!(
                "Node '{}' is not a task",
                task_name
            )))
        }
    }
}
