use clap::Parser;
use std::error::Error;

use bodo::{
    config::load_bodo_config, BodoCli, EnvManager, PluginManager, PromptManager, TaskGraph,
    TaskManager,
};

fn main() -> Result<(), Box<dyn Error>> {
    let cli = BodoCli::new();
    let config = load_bodo_config()?;

    // Initialize components
    let env_manager = EnvManager::new();
    let task_graph = TaskGraph::new();
    let plugin_manager = PluginManager::new(&config);
    let prompt_manager = PromptManager::new();

    // Create task manager
    let mut task_manager = TaskManager::new(
        &config,
        env_manager,
        task_graph,
        plugin_manager,
        prompt_manager,
    );

    task_manager.run_task(&cli.task)?;

    Ok(())
}
