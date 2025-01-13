use crate::config::{load_script_config, ColorSpec, ConcurrentItem, OutputConfig, TaskConfig};
use crate::env::EnvManager;
use crate::plugin::print_command_plugin::PrintCommandPlugin;
use crate::plugin::PluginManager;
use crate::prompt::PromptManager;
use colored::{ColoredString, Colorize};
use std::error::Error;
use std::fmt;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::thread;

#[derive(Debug)]
struct TaskError {
    message: String,
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for TaskError {}

fn get_color_for_label(label: &str) -> ColoredString {
    let colors = ["blue", "green", "yellow", "red", "magenta", "cyan"];
    let color_index = label
        .chars()
        .fold(0usize, |acc, c| (acc + c as usize) % colors.len());

    match colors[color_index] {
        "blue" => label.blue(),
        "green" => label.green(),
        "yellow" => label.yellow(),
        "red" => label.red(),
        "magenta" => label.magenta(),
        "cyan" => label.cyan(),
        _ => label.green(),
    }
}

fn apply_color(text: &str, color_spec: Option<&ColorSpec>) -> ColoredString {
    if let Some(color) = color_spec {
        match color {
            ColorSpec::Black => text.black(),
            ColorSpec::Red => text.red(),
            ColorSpec::Green => text.green(),
            ColorSpec::Yellow => text.yellow(),
            ColorSpec::Blue => text.blue(),
            ColorSpec::Magenta => text.magenta(),
            ColorSpec::Cyan => text.cyan(),
            ColorSpec::White => text.white(),
            ColorSpec::BrightBlack => text.bright_black(),
            ColorSpec::BrightRed => text.bright_red(),
            ColorSpec::BrightGreen => text.bright_green(),
            ColorSpec::BrightYellow => text.bright_yellow(),
            ColorSpec::BrightBlue => text.bright_blue(),
            ColorSpec::BrightMagenta => text.bright_magenta(),
            ColorSpec::BrightCyan => text.bright_cyan(),
            ColorSpec::BrightWhite => text.bright_white(),
        }
    } else {
        get_color_for_label(text)
    }
}

/// Holds the user config for printing output. We gather this
/// from `TaskConfig.output` if available, or fallback if not.
#[derive(Debug, Clone)]
struct PrefixSettings {
    prefix: String,
    color: Option<ColorSpec>,
    padding_width: usize,
}

pub struct TaskManager {
    pub config: TaskConfig,
    pub(crate) plugin_manager: PluginManager,
}

impl TaskManager {
    pub fn new(
        config: TaskConfig,
        _env_manager: EnvManager,
        plugin_manager: PluginManager,
        _prompt_manager: PromptManager,
    ) -> Self {
        Self {
            config,
            plugin_manager,
        }
    }

    pub fn run_task(&mut self, task_name: &str) -> Result<(), Box<dyn Error>> {
        self.plugin_manager.on_before_task_run(task_name)?;

        if let Some(command) = &self.config.command {
            let output_config = self.config.output.clone();
            self.spawn_and_wait(command, task_name, output_config)?;
        }

        Ok(())
    }

    /// Spawns the given command, prefixing all output lines
    /// with `[prefix]` in the desired color (if any).
    fn spawn_and_wait(
        &self,
        command: &str,
        task_key: &str,
        output_config: Option<OutputConfig>,
    ) -> Result<(), Box<dyn Error>> {
        // Prepare the prefix and optional color
        let prefix_settings = self.compute_prefix_settings(task_key, command, output_config);

        // Let plugins know
        self.plugin_manager.on_command_ready(command, task_key)?;

        // Spawn process
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(command);
        // Grab environment from the config if needed
        if let Some(env_vars) = &self.config.env {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }

        // Make sure we can read output
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| {
            let boxed: Box<dyn Error> = Box::new(e);
            boxed
        })?;

        // Move prefix settings into arcs so each thread can read them
        let shared_prefix = Arc::new(prefix_settings);

        // Handle stdout
        let stdout_handle = if let Some(stdout) = child.stdout.take() {
            let sp = Arc::clone(&shared_prefix);
            let task_for_err = task_key.to_string();
            Some(thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    match line {
                        Ok(line) => {
                            let prefix = format!("[{}]", sp.prefix);
                            let prefix_colored = apply_color(&prefix, sp.color.as_ref());
                            println!(
                                "{:<width$}{}",
                                prefix_colored,
                                line,
                                width = sp.padding_width
                            );
                        }
                        Err(e) => {
                            eprintln!(
                                "[BODO] Error reading stdout of task {}: {}",
                                task_for_err, e
                            );
                            break;
                        }
                    }
                }
            }))
        } else {
            None
        };

        // Handle stderr
        let stderr_handle = if let Some(stderr) = child.stderr.take() {
            let sp = Arc::clone(&shared_prefix);
            let task_for_err = task_key.to_string();
            Some(thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    match line {
                        Ok(line) => {
                            let prefix = format!("[{}]", sp.prefix);
                            let prefix_colored = apply_color(&prefix, sp.color.as_ref());
                            eprintln!(
                                "{:<width$}{}",
                                prefix_colored,
                                line,
                                width = sp.padding_width
                            );
                        }
                        Err(e) => {
                            eprintln!(
                                "[BODO] Error reading stderr of task {}: {}",
                                task_for_err, e
                            );
                            break;
                        }
                    }
                }
            }))
        } else {
            None
        };

        // Wait for process to exit
        let status = child.wait()?;
        // Wait for our I/O threads
        if let Some(handle) = stdout_handle {
            let _ = handle.join();
        }
        if let Some(handle) = stderr_handle {
            let _ = handle.join();
        }

        // We let the plugin know the result
        self.plugin_manager
            .on_after_task_run(task_key, status.code().unwrap_or(1))?;

        // Check success
        if !status.success() {
            let err = TaskError {
                message: format!(
                    "Task '{}' failed with exit code {}",
                    task_key,
                    status.code().unwrap_or(1)
                ),
            };
            self.plugin_manager.on_error(task_key, &err)?;
            return Err(Box::new(err));
        }

        Ok(())
    }

    pub fn run_concurrently(&mut self, parent_task_name: &str) -> Result<(), Box<dyn Error>> {
        if let Some(concurrent_items) = &self.config.concurrently {
            let current_script_config = load_script_config(parent_task_name)?;

            // Compute and store padding width for all concurrent items
            let (group, _) = parent_task_name
                .split_once(':')
                .unwrap_or((parent_task_name, ""));
            PrintCommandPlugin::get_padding_width(concurrent_items, group);

            let mut children = Vec::new();
            let mut command_number = 1;

            for item in concurrent_items {
                match item {
                    ConcurrentItem::Task { task, output } => {
                        let task_config = if task.contains(':') {
                            let parts: Vec<&str> = task.split(':').collect();
                            if parts.len() != 2 {
                                return Err(
                                    format!("Invalid task reference format: {}", task).into()
                                );
                            }
                            let script_config = load_script_config(parts[0])?;
                            if let Some(tasks) = &script_config.tasks {
                                tasks.get(parts[1]).cloned().ok_or_else(|| {
                                    format!(
                                        "Task '{}' not found in script '{}'",
                                        parts[1], parts[0]
                                    )
                                })?
                            } else {
                                return Err(
                                    format!("No tasks defined in script '{}'", parts[0]).into()
                                );
                            }
                        } else if let Some(tasks) = &current_script_config.tasks {
                            if let Some(task_config) = tasks.get(task) {
                                task_config.clone()
                            } else {
                                let script_config = load_script_config(task)?;
                                script_config.default_task
                            }
                        } else {
                            let script_config = load_script_config(task)?;
                            script_config.default_task
                        };

                        if let Some(command) = task_config.command {
                            let subtask_name = format!("{}:{}", parent_task_name, task);
                            self.plugin_manager
                                .on_command_ready(&command, &subtask_name)?;
                            let child = self.spawn_command_concurrent(
                                &command,
                                &subtask_name,
                                output.clone(),
                            )?;
                            children.push((child, subtask_name));
                        }
                    }
                    ConcurrentItem::Command {
                        command,
                        output,
                        name,
                    } => {
                        let command_name = if let Some(name) = name {
                            format!("{}:{}", parent_task_name, name)
                        } else {
                            let name = format!("{}:command{}", parent_task_name, command_number);
                            command_number += 1;
                            name
                        };
                        self.plugin_manager
                            .on_command_ready(command, &command_name)?;
                        let child =
                            self.spawn_command_concurrent(command, &command_name, output.clone())?;
                        children.push((child, command_name));
                    }
                }
            }

            for (mut child, subtask_name) in children {
                let status = child.wait()?;
                if !status.success() {
                    let error = TaskError {
                        message: format!(
                            "Task '{}' failed. All concurrent tasks have been stopped.",
                            subtask_name
                        ),
                    };
                    self.plugin_manager.on_error(&subtask_name, &error)?;
                    return Err(Box::new(error));
                }
            }
        }

        Ok(())
    }

    fn spawn_command_concurrent(
        &self,
        command: &str,
        task_key: &str,
        output_config: Option<OutputConfig>,
    ) -> Result<Child, Box<dyn Error>> {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(command);

        // Make sure we can read output
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn()?;

        // Prepare the prefix and optional color
        let prefix_settings = self.compute_prefix_settings(task_key, command, output_config);
        let shared_prefix = Arc::new(prefix_settings);

        // Handle stdout
        if let Some(stdout) = child.stdout.take() {
            let sp = Arc::clone(&shared_prefix);
            let task_for_err = task_key.to_string();
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    match line {
                        Ok(line) => {
                            let prefix = format!("[{}]", sp.prefix);
                            let prefix_colored = apply_color(&prefix, sp.color.as_ref());
                            println!(
                                "{:<width$}{}",
                                prefix_colored,
                                line,
                                width = sp.padding_width
                            );
                        }
                        Err(e) => {
                            eprintln!(
                                "[BODO] Error reading stdout of task {}: {}",
                                task_for_err, e
                            );
                            break;
                        }
                    }
                }
            });
        }

        // Handle stderr
        if let Some(stderr) = child.stderr.take() {
            let sp = Arc::clone(&shared_prefix);
            let task_for_err = task_key.to_string();
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    match line {
                        Ok(line) => {
                            let prefix = format!("[{}]", sp.prefix);
                            let prefix_colored = apply_color(&prefix, sp.color.as_ref());
                            eprintln!(
                                "{:<width$}{}",
                                prefix_colored,
                                line,
                                width = sp.padding_width
                            );
                        }
                        Err(e) => {
                            eprintln!(
                                "[BODO] Error reading stderr of task {}: {}",
                                task_for_err, e
                            );
                            break;
                        }
                    }
                }
            });
        }

        Ok(child)
    }

    /// Build the final prefix settings from the config, or fallback if missing.
    fn compute_prefix_settings(
        &self,
        task_key: &str,
        command: &str,
        output_config: Option<OutputConfig>,
    ) -> PrefixSettings {
        let color = output_config.as_ref().and_then(|o| o.color.clone());
        let prefix_str = if let Some(o) = output_config {
            o.prefix.unwrap_or_else(|| task_key.to_string())
        } else if task_key.contains(':') {
            // For subtasks, use the full task reference
            task_key.to_string()
        } else if task_key.ends_with(":command") {
            // For raw commands, use the command text
            command.to_string()
        } else {
            // Default to task key
            task_key.to_string()
        };

        let padding_width = PrintCommandPlugin::get_stored_padding_width();

        PrefixSettings {
            prefix: prefix_str,
            color,
            padding_width,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::EnvManager;
    use crate::prompt::PromptManager;

    #[test]
    fn test_task_manager_creation() {
        let config = TaskConfig {
            command: Some(String::from("echo test")),
            cwd: None,
            env: None,
            dependencies: Some(Vec::new()),
            plugins: None,
            concurrently: None,
            description: None,
            silent: None,
            output: None,
        };
        let env_manager = EnvManager::new();
        let plugin_manager = PluginManager::new();
        let prompt_manager = PromptManager::new();

        let _task_manager = TaskManager::new(config, env_manager, plugin_manager, prompt_manager);
    }
}
