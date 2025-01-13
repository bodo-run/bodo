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
    let prompt_manager = PromptManager::new();

    // Initialize plugins
    let mut plugin_manager = PluginManager::new(config.clone());
    plugin_manager.on_bodo_init();

    // Create task manager
    let mut task_manager = TaskManager::new(
        config,
        env_manager,
        task_graph,
        plugin_manager,
        prompt_manager,
    );

    // Run the task
    let result = task_manager.run_task(&cli.task);

    // Clean up plugins
    match &result {
        Ok(_) => task_manager.cleanup(0),
        Err(_) => task_manager.cleanup(1),
    }

    result
}
