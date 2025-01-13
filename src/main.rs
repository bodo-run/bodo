use bodo::config::load_script_config;
use bodo::env::EnvManager;
use bodo::plugin::PluginManager;
use bodo::prompt::PromptManager;
use bodo::task::TaskManager;
use std::error::Error;
use std::env;

fn main() -> Result<(), Box<dyn Error>> {
    // Get command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err("Usage: bodo <task_name>".into());
    }
    let task_name = &args[1];

    // Initialize managers
    let env_manager = EnvManager::new();
    let plugin_manager = PluginManager::new();
    let prompt_manager = PromptManager::new();

    // Load script config
    let script_config = load_script_config(task_name)?;
    let task_config = script_config.default_task;

    // Create task manager
    let mut task_manager =
        TaskManager::new(task_config, env_manager, plugin_manager, prompt_manager);

    // Run task
    match task_manager.run_task(task_name) {
        Ok(_) => task_manager.on_exit(0),
        Err(_) => task_manager.on_exit(1),
    }
}
