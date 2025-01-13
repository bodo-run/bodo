use bodo::cli::BodoCli;
use bodo::config::{load_bodo_config, load_script_config, TaskConfig};
use bodo::debug;
use bodo::env::EnvManager;
use bodo::plugin::PluginManager;
use bodo::prompt::PromptManager;
use bodo::task::TaskManager;
use bodo::watch::WatchManager;
use clap::Parser;
use colored::*;
use std::collections::HashSet;
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
                .cloned()
        } else {
            Err("No subtasks defined".into())
        }
    } else {
        Ok(script_config.default_task.clone())
    }
}

fn list_tasks(script_name: Option<&str>) -> Result<(), Box<dyn Error>> {
    // Find all script directories
    let current_dir = std::env::current_dir()?;
    let scripts_dir = current_dir.join("scripts");
    if !scripts_dir.exists() {
        println!(
            "\n{}",
            "No tasks found. Create a script.yaml file in:".yellow()
        );
        println!(
            "  {}/scripts/<task-name>/script.yaml",
            current_dir.display()
        );
        return Ok(());
    }

    println!("\n{}", "Available Tasks:".bold().green());
    println!();

    if let Some(name) = script_name {
        let script_path = scripts_dir.join(name).join("script.yaml");
        if script_path.exists() {
            let script_config = load_script_config(name)?;

            // Print task description
            if let Some(desc) = script_config.description {
                println!("{}", desc);
            }

            // Print default task
            println!("default:");
            if let Some(cmd) = script_config.default_task.command {
                println!("  $ {}", cmd);
            }
            println!();

            // Print subtasks
            if let Some(subtasks) = script_config.subtasks {
                for (name, task) in subtasks {
                    if let Some(desc) = task.description {
                        println!("{}:", name);
                        println!("  {}", desc);
                    } else {
                        println!("{}:", name);
                    }
                    if let Some(cmd) = task.command {
                        println!("  $ {}", cmd);
                    }
                    println!();
                }
            }
        }
        return Ok(());
    }

    // List all task directories if no specific script is requested
    for entry in std::fs::read_dir(&scripts_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let task_name = entry.file_name();
            let script_path = entry.path().join("script.yaml");
            if script_path.exists() {
                let script_config = load_script_config(&task_name.to_string_lossy())?;

                // Print task name and description
                println!("\n{}", task_name.to_string_lossy().yellow());
                if let Some(desc) = script_config.description {
                    println!("  {}", desc);
                }

                // Print default task
                println!("  {}", "default:".bold());
                if let Some(desc) = script_config.default_task.description {
                    println!("    {}", desc);
                }
                if let Some(cmd) = script_config.default_task.command {
                    println!("    {}", format!("$ {}", cmd).dimmed());
                }

                // Print subtasks
                if let Some(subtasks) = script_config.subtasks {
                    for (name, task) in subtasks {
                        println!("\n  {}:", name.bold());
                        if let Some(desc) = task.description {
                            println!("    {}", desc);
                        }
                        if let Some(cmd) = task.command {
                            println!("    {}", format!("$ {}", cmd).dimmed());
                        }
                    }
                }
            }
        }
    }
    println!();
    Ok(())
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
    let task_config = get_task_config(script_config, subtask)?;
    let task_key = match subtask {
        Some(s) => format!("{}:{}", task_name, s),
        None => task_name.to_string(),
    };

    if !visited.insert(task_key.clone()) {
        return Err(format!("Circular dependency detected for task '{}'", task_key).into());
    }

    // Resolve dependencies
    if let Some(deps) = &task_config.dependencies {
        for dep in deps {
            let parts: Vec<&str> = dep.split(':').collect();
            match parts.as_slice() {
                [task, subtask] => {
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

    let mut task_manager =
        TaskManager::new(task_config, env_manager, plugin_manager, prompt_manager);

    if task_manager.config.concurrently.is_some() {
        task_manager.run_concurrently(&task_key)?;
    } else {
        task_manager.run_task(&task_key)?;
    }

    visited.remove(&task_key);

    Ok(())
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
    let cli = BodoCli::parse();

    // Set verbose mode
    debug::set_verbose(cli.verbose);

    if cli.list {
        return list_tasks(cli.task.as_deref());
    }

    let task_name = cli
        .task
        .ok_or("No task specified. Use --list to see available tasks.")?;
    let subtask = if !cli.args.is_empty() {
        Some(cli.args[0].as_str())
    } else {
        None
    };

    // Initialize managers
    let mut env_manager = EnvManager::new();
    let mut plugin_manager = PluginManager::new();
    plugin_manager.set_verbose(cli.verbose);
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
    let script_config = load_script_config(&task_name)?;

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

    // Get task config
    let task_config = get_task_config(&script_config, subtask)?;

    // Create task manager
    let task_manager = TaskManager::new(
        task_config,
        env_manager.clone(),
        plugin_manager.clone(),
        prompt_manager.clone(),
    );

    // Run the task
    if cli.watch {
        let watch_manager = WatchManager::new(task_manager);
        watch_manager.watch_and_run(&task_name, subtask)?;
    } else {
        run_task(
            &task_name,
            subtask,
            env_manager,
            plugin_manager,
            prompt_manager,
            &script_config,
        )?;
    }

    Ok(())
}
