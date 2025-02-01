use crate::config::Dependency;
use crate::config::TaskConfig;
use crate::plugin::PluginManager;
use colored::{ColoredString, Colorize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

#[derive(Debug, Clone)]
pub enum ColorSpec {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

#[derive(Debug, Clone)]
pub struct OutputConfig {
    pub prefix: Option<String>,
    pub color: Option<ColorSpec>,
    pub disable_color: Option<bool>,
}

#[derive(Debug, Clone)]
pub enum ConcurrentItem {
    Task {
        task: String,
        output: Option<OutputConfig>,
    },
    Command {
        command: String,
        name: Option<String>,
        output: Option<OutputConfig>,
    },
}

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

#[derive(Debug, Clone)]
pub enum NodeKind {
    Task(TaskData),
    Command(CommandData),
    ConcurrentGroup(ConcurrentGroupData),
}

#[derive(Debug, Clone)]
pub struct TaskData {
    pub name: String,
    pub command: Option<String>,
    pub working_dir: Option<String>,
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct CommandData {
    pub raw_command: String,
}

#[derive(Debug, Clone)]
pub struct ConcurrentGroupData {
    pub fail_fast: bool,
    pub max_concurrent: Option<usize>,
    pub items: Vec<Dependency>,
}

pub struct TaskManager<'a> {
    pub config: TaskConfig,
    // allow dead code for now
    #[allow(dead_code)]
    pub(crate) plugin_manager: &'a PluginManager,
}

impl<'a> TaskManager<'a> {
    pub fn new(config: TaskConfig, plugin_manager: &'a PluginManager) -> Self {
        Self {
            config,
            plugin_manager,
        }
    }

    pub fn run_task(&self, task_name: &str) -> Result<(), Box<dyn Error>> {
        // Get the node from the registry
        let node = self.get_node(task_name)?;

        match &node {
            NodeKind::Task(task_data) => {
                self.run_single_task(task_data)?;
            }
            NodeKind::Command(cmd_data) => {
                self.run_single_command(cmd_data)?;
            }
            NodeKind::ConcurrentGroup(group_data) => {
                self.run_concurrent_group(group_data)?;
            }
        }
        Ok(())
    }

    fn get_node(&self, task_name: &str) -> Result<NodeKind, Box<dyn Error>> {
        // For now, simulate node retrieval based on config
        // This should be replaced with actual graph node lookup
        if !self.config.concurrently.is_empty() {
            Ok(NodeKind::ConcurrentGroup(ConcurrentGroupData {
                fail_fast: self.config.concurrently_options.fail_fast.unwrap_or(false),
                max_concurrent: self.config.concurrently_options.max_concurrent_tasks,
                items: self.config.concurrently.clone(),
            }))
        } else if let Some(cmd) = &self.config.command {
            Ok(NodeKind::Task(TaskData {
                name: task_name.to_string(),
                command: Some(cmd.clone()),
                working_dir: self.config.cwd.clone(),
                env: self.config.env.clone(),
            }))
        } else {
            Err(Box::new(TaskError(format!(
                "Task '{}' not found",
                task_name
            ))))
        }
    }

    fn run_single_task(&self, task_data: &TaskData) -> Result<(), Box<dyn Error>> {
        if let Some(command) = &task_data.command {
            self.spawn_and_wait(command, &task_data.name, None)?;
        }
        Ok(())
    }

    fn run_single_command(&self, cmd_data: &CommandData) -> Result<(), Box<dyn Error>> {
        self.spawn_and_wait(&cmd_data.raw_command, "command", None)
    }

    fn run_concurrent_group(&self, group: &ConcurrentGroupData) -> Result<(), Box<dyn Error>> {
        run_in_parallel(&group.items, Some(group.fail_fast))
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

        // Spawn process
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(command);

        // Set working directory if specified
        if let Some(cwd) = &self.config.cwd {
            cmd.current_dir(cwd);
        }

        // Grab environment from the config if needed
        for (key, value) in &self.config.env {
            cmd.env(key, value);
        }

        // Make sure we can read output
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| {
            let boxed: Box<dyn Error> = Box::new(e);
            boxed
        })?;

        // Move prefix settings into arcs so each thread can read them
        let prefix_settings = Arc::new(prefix_settings);
        let task_key = task_key.to_string();

        // Handle stdout
        let stdout_handle = if let Some(stdout) = child.stdout.take() {
            let prefix_settings = Arc::clone(&prefix_settings);
            let task_key = task_key.clone();
            Some(thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    match line {
                        Ok(line) => {
                            let prefix = format!("[{}]", prefix_settings.prefix);
                            let prefix_colored =
                                apply_color(&prefix, prefix_settings.color.as_ref());
                            println!(
                                "{:<width$}{}",
                                prefix_colored,
                                line,
                                width = prefix_settings.padding_width
                            );
                        }
                        Err(e) => {
                            eprintln!("[BODO] Error reading stdout of task {}: {}", task_key, e);
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
            let prefix_settings = Arc::clone(&prefix_settings);
            let task_key = task_key.clone();
            Some(thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    match line {
                        Ok(line) => {
                            let prefix = format!("[{}]", prefix_settings.prefix);
                            let prefix_colored =
                                apply_color(&prefix, prefix_settings.color.as_ref());
                            eprintln!(
                                "{:<width$}{}",
                                prefix_colored,
                                line,
                                width = prefix_settings.padding_width
                            );
                        }
                        Err(e) => {
                            eprintln!("[BODO] Error reading stderr of task {}: {}", task_key, e);
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

        // Check success
        if !status.success() {
            let err = TaskError(format!(
                "Task '{}' failed with exit code {}",
                task_key,
                status.code().unwrap_or(1)
            ));
            return Err(Box::new(err));
        }

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

        PrefixSettings {
            prefix: prefix_str,
            color,
            padding_width: 20,
        }
    }
}

fn run_in_parallel(tasks: &[Dependency], fail_fast: Option<bool>) -> Result<(), Box<dyn Error>> {
    let mut handles = Vec::new();
    let fail_fast = fail_fast.unwrap_or(false);

    for task in tasks {
        match task {
            Dependency::Command { command } => {
                let command = command.clone();
                let handle =
                    thread::spawn(move || Command::new("sh").arg("-c").arg(&command).status());
                handles.push(handle);
            }
            Dependency::Task { task } => {
                // For now, just print that we would run the task
                println!("Would run task: {}", task);
            }
        }
    }

    for handle in handles {
        if let Ok(status) = handle.join().unwrap() {
            if !status.success() && fail_fast {
                return Err(Box::new(TaskError("A concurrent task failed".to_string())));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_task_manager_creation() {
        let config = TaskConfig {
            description: None,
            command: Some(String::from("echo test")),
            cwd: None,
            pre_deps: Vec::new(),
            post_deps: Vec::new(),
            watch: None,
            timeout: None,
            env: HashMap::new(),
            concurrently_options: Default::default(),
            concurrently: Vec::new(),
        };
        let plugin_manager = PluginManager::new();

        let _task_manager = TaskManager::new(config, &plugin_manager);
    }
}
