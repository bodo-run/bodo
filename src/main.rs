use bodo::config::{load_bodo_config, load_script_config, TaskConfig};
use bodo::env::EnvManager;
use bodo::plugin::PluginManager;
use bodo::prompt::PromptManager;
use bodo::task::TaskManager;
use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::path::PathBuf;

fn get_task_config(
    script_config: &bodo::config::ScriptConfig,
    subtask: Option<&str>,
) -> Result<TaskConfig, Box<dyn Error>> {
    if let Some(subtask_name) = subtask {
        if let Some(subtasks) = &script_config.subtasks {
            subtasks
                .get(subtask_name)
                .ok_or_else(|| {
                    Box::<dyn Error>::from(format!("Subtask '{}' not found", subtask_name))
                })
                .map(|t| t.clone())
        } else {
            Err("No subtasks defined".into())
        }
    } else {
        Ok(script_config.default_task.clone())
    }
}

fn run_task_with_deps(
    task_name: &str,
    subtask: Option<&str>,
    env_manager: EnvManager,
    plugin_manager: PluginManager,
    prompt_manager: PromptManager,
    script_config: &bodo::config::ScriptConfig,
    visited: &mut HashSet<String>,
) -> Result<(), Box<dyn Error>> {
    // Get task config based on whether a subtask was specified
    let task_config = get_task_config(script_config, subtask)?;

    // Check for circular dependencies
    let task_key = match subtask {
        Some(s) => format!("{}:{}", task_name, s),
        None => task_name.to_string(),
    };

    if !visited.insert(task_key.clone()) {
        return Err(format!("Circular dependency detected for task '{}'", task_key).into());
    }

    // If task has dependencies, run them first
    if let Some(deps) = &task_config.dependencies {
        for dep in deps {
            // Parse task path (format: "task:subtask" or "task")
            let parts: Vec<&str> = dep.split(':').collect();
            match parts.as_slice() {
                [task, subtask] => {
                    // External dependency
                    let dep_script_config = load_script_config(task)?;
                    run_task_with_deps(
                        task,
                        Some(subtask),
                        env_manager.clone(),
                        plugin_manager.clone(),
                        prompt_manager.clone(),
                        &dep_script_config,
                        visited,
                    )?;
                }
                [task] => {
                    // Local dependency (subtask)
                    run_task_with_deps(
                        task_name,
                        Some(task),
                        env_manager.clone(),
                        plugin_manager.clone(),
                        prompt_manager.clone(),
                        script_config,
                        visited,
                    )?;
                }
                _ => return Err(format!("Invalid dependency format: {}", dep).into()),
            }
        }
    }

    // Create task manager for this task
    let mut task_manager =
        TaskManager::new(task_config, env_manager, plugin_manager, prompt_manager);

    // Run the task
    let result = task_manager.run_task(&task_key);

    // Remove the task from visited after execution
    visited.remove(&task_key);

    result
}

fn run_task(
    task_name: &str,
    subtask: Option<&str>,
    env_manager: EnvManager,
    plugin_manager: PluginManager,
    prompt_manager: PromptManager,
    script_config: &bodo::config::ScriptConfig,
) -> Result<(), Box<dyn Error>> {
    let mut visited = HashSet::new();
    run_task_with_deps(
        task_name,
        subtask,
        env_manager,
        plugin_manager,
        prompt_manager,
        script_config,
        &mut visited,
    )
}

fn main() -> Result<(), Box<dyn Error>> {
    // Get command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err("Usage: bodo <task_name> [subtask] or bodo watch <task_name> [subtask]".into());
    }

    // Parse arguments based on command type
    let (task_name, subtask) = if args[1] == "watch" {
        if args.len() < 3 {
            return Err("Usage: bodo watch <task_name> [subtask]".into());
        }
        (&args[2], args.get(3).map(|s| s.as_str()))
    } else {
        (&args[1], args.get(2).map(|s| s.as_str()))
    };

    // Initialize managers
    let mut env_manager = EnvManager::new();
    let mut plugin_manager = PluginManager::new();
    let prompt_manager = PromptManager::new();

    // Load global bodo config
    let bodo_config = load_bodo_config()?;

    // Register plugins from bodo.yaml
    if let Some(plugins) = bodo_config.plugins {
        for plugin_path in plugins {
            plugin_manager.register_plugin(PathBuf::from(&plugin_path));
        }
    }

    // Load script config for environment setup
    let script_config = load_script_config(task_name)?;

    // Load environment variables from script config
    if let Some(env_vars) = &script_config.env {
        for (key, value) in env_vars {
            std::env::set_var(key, value);
        }
    }

    // Load exec paths if present
    if let Some(exec_paths) = &script_config.exec_paths {
        env_manager.inject_exec_paths(exec_paths);
    }

    // Run the task and handle exit
    match run_task(
        task_name,
        subtask,
        env_manager,
        plugin_manager,
        prompt_manager,
        &script_config,
    ) {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Error: {:?}", e);
            std::process::exit(1);
        }
    }
}
