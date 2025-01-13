use bodo::config::TaskConfig;
use bodo::env::EnvManager;
use bodo::plugin::PluginManager;
use bodo::prompt::PromptManager;
use bodo::task::TaskManager;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize managers
    let env_manager = EnvManager::new();
    let plugin_manager = PluginManager::new();
    let prompt_manager = PromptManager::new();

    // Create task config
    let task_config = TaskConfig {
        command: String::from("echo test"),
        cwd: None,
        env: None,
        dependencies: Some(Vec::new()),
        plugins: None,
    };

    // Create task manager
    let mut task_manager =
        TaskManager::new(task_config, env_manager, plugin_manager, prompt_manager);

    // Run task
    match task_manager.run_task("test") {
        Ok(_) => task_manager.on_exit(0),
        Err(_) => task_manager.on_exit(1),
    }
}
