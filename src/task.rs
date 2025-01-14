use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use crate::config::{
    load_bodo_config, load_script_config, ColorSpec, ConcurrentItem, OutputConfig, TaskConfig,
};
use crate::env::EnvManager;
use crate::plugin::print_command_plugin::PrintCommandPlugin;
use crate::plugin::PluginManager;
use crate::process::ProcessManager;
use crate::prompt::PromptManager;
use colored::{ColoredString, Colorize};

pub struct ConcurrentChild {
    pub child: Child,
    pub stdout_handle: Option<JoinHandle<()>>,
    pub stderr_handle: Option<JoinHandle<()>>,
}

impl ConcurrentChild {
    pub fn wait(&mut self) -> std::io::Result<ExitStatus> {
        let status = self.child.wait()?;
        // Join stdout/stderr threads after process exits
        if let Some(handle) = self.stdout_handle.take() {
            let _ = handle.join();
        }
        if let Some(handle) = self.stderr_handle.take() {
            let _ = handle.join();
        }
        Ok(status)
    }

    pub fn kill(&mut self) -> Result<(), Box<dyn Error>> {
        // First close stdout/stderr to prevent any more output
        if let Some(handle) = self.stdout_handle.take() {
            let _ = handle.join();
        }
        if let Some(handle) = self.stderr_handle.take() {
            let _ = handle.join();
        }
        // Then kill the process
        let _ = self.child.kill();
        // Wait for it to ensure it's dead
        let _ = self.child.wait();
        Ok(())
    }
}

#[derive(Debug)]
struct TaskError(String);

impl std::error::Error for TaskError {}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

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
        mut config: TaskConfig,
        _env_manager: EnvManager,
        plugin_manager: PluginManager,
        _prompt_manager: PromptManager,
    ) -> Self {
        // If we have a script path, load the script config
        if let Ok(script_config) = load_script_config("") {
            // Merge top-level tasks into config so concurrency sees them
            if let Some(script_tasks) = script_config.tasks {
                if config.tasks.is_none() {
                    config.tasks = Some(HashMap::new());
                }
                if let Some(ref mut existing) = config.tasks {
                    for (k, v) in script_tasks {
                        existing.insert(k, v);
                    }
                }
            }
        }

        Self {
            config,
            plugin_manager,
        }
    }

    pub fn run_task(&mut self, task_name: &str) -> Result<(), Box<dyn Error>> {
        self.plugin_manager.on_before_task_run(task_name)?;

        if let Some(command) = &self.config.command {
            let output_config = self.config.output.clone();
            self.plugin_manager.on_command_ready(command, task_name)?;
            self.spawn_and_wait(command, task_name, output_config)?;
        } else if let Some(items) = &self.config.concurrently {
            self.run_concurrently(items.clone(), task_name)?;
        } else {
            // Try to load the task from the script config
            if let Ok(script_config) = load_script_config(task_name) {
                let mut merged_config = script_config.default_task;
                // Copy over the top-level tasks so concurrency can find them
                merged_config.tasks = script_config.tasks.clone();
                // Also merge any tasks from the current config
                if let Some(current_tasks) = &self.config.tasks {
                    if let Some(merged_tasks) = &mut merged_config.tasks {
                        for (key, value) in current_tasks {
                            merged_tasks.insert(key.clone(), value.clone());
                        }
                    }
                }
                let env_manager = EnvManager::new();
                let prompt_manager = PromptManager::new();
                let mut task_manager = TaskManager::new(
                    merged_config,
                    env_manager,
                    self.plugin_manager.clone(),
                    prompt_manager,
                );
                task_manager.run_task(task_name)?;
            }
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
        // Get color settings from all config levels
        let global_disable = load_bodo_config().ok().and_then(|c| c.disable_color);

        let script_disable = if let Some((group, _)) = task_key.split_once(':') {
            load_script_config(group).ok().and_then(|c| c.disable_color)
        } else {
            load_script_config(task_key)
                .ok()
                .and_then(|c| c.disable_color)
        };

        let task_disable = self.config.disable_color;
        let output_disable = output_config.as_ref().and_then(|o| o.disable_color);

        let _color_enabled = is_color_enabled(
            &global_disable,
            &script_disable,
            &task_disable,
            &output_disable,
        );

        // Prepare the prefix and optional color
        let prefix_settings = self.compute_prefix_settings(task_key, command, output_config);
        let is_concurrent = self.config.concurrently.is_some();

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
        let shared_is_concurrent = Arc::new(is_concurrent);

        // Handle stdout
        let stdout_handle = if let Some(stdout) = child.stdout.take() {
            let sp = Arc::clone(&shared_prefix);
            let sic = Arc::clone(&shared_is_concurrent);
            let task_for_err = task_key.to_string();
            Some(thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    match line {
                        Ok(line) => {
                            if *sic {
                                let prefix = format!("[{}]", sp.prefix);
                                let prefix_colored = apply_color(&prefix, sp.color.as_ref());
                                println!(
                                    "{:<width$}{}",
                                    prefix_colored,
                                    line,
                                    width = sp.padding_width
                                );
                            } else {
                                println!("{}", line);
                            }
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
            let sic = Arc::clone(&shared_is_concurrent);
            let task_for_err = task_key.to_string();
            Some(thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    match line {
                        Ok(line) => {
                            if *sic {
                                let prefix = format!("[{}]", sp.prefix);
                                let prefix_colored = apply_color(&prefix, sp.color.as_ref());
                                eprintln!(
                                    "{:<width$}{}",
                                    prefix_colored,
                                    line,
                                    width = sp.padding_width
                                );
                            } else {
                                eprintln!("{}", line);
                            }
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
            let err = TaskError(format!(
                "Task '{}' failed with exit code {}",
                task_key,
                status.code().unwrap_or(1)
            ));
            self.plugin_manager.on_error(task_key, &err)?;
            return Err(Box::new(err));
        }

        Ok(())
    }

    pub fn get_task_config(&self, task_key: &str) -> Option<TaskConfig> {
        if task_key.contains(':') {
            let parts: Vec<&str> = task_key.split(':').collect();
            if parts.len() != 2 {
                return None;
            }
            if let Ok(script_config) = load_script_config(parts[0]) {
                if let Some(tasks) = &script_config.tasks {
                    return tasks.get(parts[1]).cloned();
                }
            }
        } else if let Ok(script_config) = load_script_config(task_key) {
            return Some(script_config.default_task);
        }
        None
    }

    pub fn run_concurrently(
        &self,
        items: Vec<ConcurrentItem>,
        parent_task_name: &str,
    ) -> Result<(), Box<dyn Error>> {
        // Gather concurrency options
        let concurrently_options = self.config.concurrently_options.clone();
        let fail_fast = concurrently_options.as_ref().is_some_and(|c| c.fail_fast);

        // Create a new process manager with the fail_fast setting
        let mut process_manager = ProcessManager::new(fail_fast);

        // Spawn each item
        for (index, item) in items.into_iter().enumerate() {
            let task_key = match &item {
                ConcurrentItem::Task { task, .. } => format!("{}:{}", parent_task_name, task),
                ConcurrentItem::Command { name, .. } => {
                    if let Some(n) = name {
                        format!("{}:{}", parent_task_name, n)
                    } else {
                        // fallback name for command
                        format!("{}:command{}", parent_task_name, index + 1)
                    }
                }
            };

            let (command, output_conf) = match item {
                ConcurrentItem::Task { task, output } => {
                    // We either find the subtask config or error
                    let subtask_config = if let Some(tasks) = &self.config.tasks {
                        tasks
                            .get(&task)
                            .cloned()
                            .or_else(|| self.get_task_config(&task))
                    } else {
                        self.get_task_config(&task)
                    };

                    let subtask_config = match subtask_config {
                        Some(c) => c,
                        None => {
                            return Err(format!("Task '{}' not found", task).into());
                        }
                    };

                    let cmd = subtask_config.command.clone().unwrap_or_default();
                    (cmd, output)
                }
                ConcurrentItem::Command {
                    command,
                    output,
                    name: _,
                } => (command, output),
            };

            // Let the plugin manager know
            self.plugin_manager.on_command_ready(&command, &task_key)?;

            // Prepare the prefix and color settings
            let prefix_settings = self.compute_prefix_settings(&task_key, &command, output_conf);

            // Spawn the process using the process manager
            process_manager.spawn_command(
                &prefix_settings.prefix,
                &command,
                prefix_settings.color,
            )?;
        }

        // Run all processes concurrently and wait for completion
        process_manager.run_concurrently()?;

        Ok(())
    }

    /// Build the final prefix settings from the config, or fallback if missing.
    fn compute_prefix_settings(
        &self,
        task_key: &str,
        command: &str,
        output_config: Option<OutputConfig>,
    ) -> PrefixSettings {
        let color = output_config.as_ref().and_then(|c| c.color.clone());

        let prefix_str = if let Some(output_config) = output_config {
            if let Some(prefix) = output_config.prefix {
                prefix
            } else if task_key.contains(':') {
                task_key.to_string()
            } else if task_key.ends_with("command") {
                command.to_string()
            } else {
                task_key.to_string()
            }
        } else if task_key.contains(':') {
            task_key.to_string()
        } else if task_key.ends_with("command") {
            command.to_string()
        } else {
            task_key.to_string()
        };

        let padding_width = PrintCommandPlugin::get_stored_padding_width();

        PrefixSettings {
            prefix: prefix_str,
            color,
            padding_width,
        }
    }

    pub fn spawn_and_wait_concurrent(
        &self,
        command: &str,
        task_key: &str,
        output_config: Option<OutputConfig>,
    ) -> Result<ConcurrentChild, Box<dyn Error>> {
        // Prepare the prefix and optional color
        let prefix_settings = self.compute_prefix_settings(task_key, command, output_config);
        let is_concurrent = self.config.concurrently.is_some();

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
        let shared_is_concurrent = Arc::new(is_concurrent);

        // Create a channel for each output stream
        let (stdout_tx, stdout_rx) = std::sync::mpsc::channel();
        let (stderr_tx, stderr_rx) = std::sync::mpsc::channel();

        // Take ownership of the output streams
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        // Spawn stdout thread
        let stdout_handle = {
            let sp = Arc::clone(&shared_prefix);
            let sic = Arc::clone(&shared_is_concurrent);
            let task_for_err = task_key.to_string();
            Some(thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    match line {
                        Ok(line) => {
                            if *sic {
                                let prefix = format!("[{}]", sp.prefix);
                                let prefix_colored = apply_color(&prefix, sp.color.as_ref());
                                let output = format!(
                                    "{:<width$}{}",
                                    prefix_colored,
                                    line,
                                    width = sp.padding_width
                                );
                                let _ = stdout_tx.send(output);
                            } else {
                                let _ = stdout_tx.send(line);
                            }
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
        };

        // Spawn stderr thread
        let stderr_handle = {
            let sp = Arc::clone(&shared_prefix);
            let sic = Arc::clone(&shared_is_concurrent);
            let task_for_err = task_key.to_string();
            Some(thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    match line {
                        Ok(line) => {
                            if *sic {
                                let prefix = format!("[{}]", sp.prefix);
                                let prefix_colored = apply_color(&prefix, sp.color.as_ref());
                                let output = format!(
                                    "{:<width$}{}",
                                    prefix_colored,
                                    line,
                                    width = sp.padding_width
                                );
                                let _ = stderr_tx.send(output);
                            } else {
                                let _ = stderr_tx.send(line);
                            }
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
        };

        // Spawn output printer threads
        thread::spawn(move || {
            while let Ok(line) = stdout_rx.recv() {
                println!("{}", line);
            }
        });

        thread::spawn(move || {
            while let Ok(line) = stderr_rx.recv() {
                eprintln!("{}", line);
            }
        });

        Ok(ConcurrentChild {
            child,
            stdout_handle,
            stderr_handle,
        })
    }
}

/// Helper function to determine if color should be enabled based on config hierarchy
fn is_color_enabled(
    global_config: &Option<bool>,
    script_config: &Option<bool>,
    task_config: &Option<bool>,
    output_config: &Option<bool>,
) -> bool {
    // Check configs in order of precedence (highest to lowest)
    // If any level explicitly disables color, return false
    if output_config.unwrap_or(false)
        || task_config.unwrap_or(false)
        || script_config.unwrap_or(false)
        || global_config.unwrap_or(false)
    {
        return false;
    }
    true
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
            concurrently_options: None,
            description: None,
            silent: None,
            output: None,
            disable_color: None,
            tasks: None,
        };
        let env_manager = EnvManager::new();
        let plugin_manager = PluginManager::new();
        let prompt_manager = PromptManager::new();

        let _task_manager = TaskManager::new(config, env_manager, plugin_manager, prompt_manager);
    }
}
