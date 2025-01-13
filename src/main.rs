use bodo::cli::BodoCli;
use bodo::config::{load_bodo_config, load_script_config, ConcurrentItem, TaskConfig};
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
use std::process;

fn get_task_config(
    script_config: &bodo::config::ScriptConfig,
    subtask: Option<&str>,
) -> Result<TaskConfig, Box<dyn Error>> {
    if let Some(subtask_name) = subtask {
        if let Some(tasks) = &script_config.tasks {
            tasks
                .get(subtask_name)
                .ok_or_else(|| {
                    Box::<dyn Error>::from(format!("Subtask '{}' not found", subtask_name))
                })
                .cloned()
        } else {
            Err("No tasks defined".into())
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

            // Print tasks
            if let Some(tasks) = script_config.tasks {
                for (name, task) in tasks {
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

                // Print tasks
                if let Some(tasks) = script_config.tasks {
                    for (name, task) in tasks {
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
            match dep {
                ConcurrentItem::Task { task, output: _ } => {
                    let parts: Vec<&str> = task.split(':').collect();
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
                        _ => return Err(format!("Invalid dependency format: {}", task).into()),
                    }
                }
                ConcurrentItem::Command {
                    command,
                    output: _,
                    name: _,
                } => {
                    let mut cmd = std::process::Command::new("sh");
                    cmd.arg("-c").arg(command);
                    let status = cmd.status()?;
                    if !status.success() {
                        return Err(format!("Command failed with status: {}", status).into());
                    }
                }
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

fn main() {
    let result = run_main();
    if let Err(err) = result {
        eprintln!("{}", err.to_string().red());
        process::exit(1);
    }
}

fn run_main() -> Result<(), Box<dyn Error>> {
    let cli = BodoCli::parse();
    debug::set_verbose(cli.verbose);

    // Handle `--list` first
    if cli.list {
        return list_tasks(cli.task.as_deref());
    }

    // If `bodo` was run with no <task>, use scripts/script.yaml
    let (task_name, subtask) = match &cli.task {
        Some(t) => {
            // First check if this is a subtask in root script
            let root_script = std::env::current_dir()?.join("scripts").join("script.yaml");
            if root_script.exists() {
                let root_config_str = std::fs::read_to_string(&root_script)?;
                let root_config: bodo::config::ScriptConfig =
                    serde_yaml::from_str(&root_config_str)?;
                if let Some(tasks) = &root_config.tasks {
                    if tasks.contains_key(t) {
                        // Check if there's also a script directory with the same name
                        let script_dir = std::env::current_dir()?.join("scripts").join(t);
                        if script_dir.exists() && script_dir.is_dir() {
                            eprintln!("{}", format!("Warning: Task '{}' exists in both root script.yaml and as a script directory. Using the root script task.", t).yellow());
                        }
                        // It's a subtask in root script
                        (".".to_string(), Some(t.as_str()))
                    } else {
                        // Not a subtask, treat as separate script
                        (t.clone(), None)
                    }
                } else {
                    // No tasks in root script, treat as separate script
                    (t.clone(), None)
                }
            } else {
                // No root script, treat as separate script
                (t.clone(), None)
            }
        }
        None => {
            let root_script = std::env::current_dir()?.join("scripts").join("script.yaml");
            if root_script.exists() {
                (".".to_string(), None)
            } else {
                return Err("No task specified and no scripts/script.yaml found. Use --list to see available tasks.".into());
            }
        }
    };

    // Additional subtask from args (only if we don't already have a subtask)
    let subtask = if subtask.is_none() && !cli.args.is_empty() {
        Some(cli.args[0].as_str())
    } else {
        subtask
    };

    // Initialize managers
    let mut env_manager = EnvManager::new();
    let mut plugin_manager = PluginManager::new();
    plugin_manager.set_verbose(cli.verbose);
    let prompt_manager = PromptManager::new();

    // Load global bodo config
    let bodo_config = load_bodo_config()?;
    if let Some(plugins) = bodo_config.plugins {
        for plugin_path in plugins {
            plugin_manager.register_plugin(PathBuf::from(&plugin_path));
        }
    }

    // Now attempt to load the script for the task
    let script_config = if task_name == "." {
        // Load root script directly
        let config_str =
            std::fs::read_to_string(std::env::current_dir()?.join("scripts").join("script.yaml"))?;
        serde_yaml::from_str(&config_str)?
    } else {
        load_script_config(&task_name)?
    };

    // Merge top-level env from script.yaml
    if let Some(env_vars) = &script_config.env {
        for (key, value) in env_vars {
            std::env::set_var(key, value);
        }
    }

    // Merge exec paths if present
    if let Some(exec_paths) = &script_config.exec_paths {
        env_manager.inject_exec_paths(exec_paths);
    }

    // Now run the user's requested task, or watch
    let task_config = get_task_config(&script_config, subtask)?;
    let task_manager = TaskManager::new(
        task_config,
        env_manager.clone(),
        plugin_manager.clone(),
        prompt_manager.clone(),
    );

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
