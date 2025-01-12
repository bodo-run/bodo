use clap::Parser;
use std::process::exit;

mod cli;
mod config;
mod env;
mod graph;
mod plugin;
mod prompt;
mod task;
mod watch;

use cli::BodoCli;
use config::BodoConfig;
use env::EnvManager;
use graph::TaskGraph;
use plugin::PluginManager;
use prompt::PromptManager;
use task::TaskManager;
use watch::WatchManager;

fn main() {
    // Parse command line arguments
    let cli = BodoCli::parse();

    // Load configuration
    let config = BodoConfig::default();

    // Initialize components
    let env_manager = EnvManager::new();
    let task_graph = TaskGraph::new();
    let plugin_manager = PluginManager::new(config.clone());
    let prompt_manager = PromptManager::new();

    // Create task manager
    let task_manager = TaskManager::new(
        config,
        env_manager,
        task_graph,
        plugin_manager,
        prompt_manager,
    );

    // Handle watch mode
    if cli.watch {
        let watch_manager = WatchManager::new(task_manager);
        if let Err(e) = watch_manager.watch_and_run(
            cli.task_group.as_deref().unwrap_or(""),
            cli.subtask.as_deref(),
        ) {
            eprintln!("Watch error: {}", e);
            exit(1);
        }
    } else {
        // Run task directly
        if let Err(e) = task_manager.run_task(
            cli.task_group.as_deref().unwrap_or(""),
            cli.subtask.as_deref(),
        ) {
            eprintln!("Task error: {}", e);
            exit(1);
        }
    }
} 