use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    errors::PluginError,
    graph::{CommandData, Graph, NodeKind, TaskData},
};

use glob::glob;

#[derive(Debug, Deserialize, Default)]
pub struct BodoConfig {
    pub script_paths: Option<Vec<String>>,
}

/// ScriptFile holds the YAML definition of tasks/commands.
#[derive(Debug, Deserialize)]
pub struct ScriptFile {
    pub name: Option<String>,
    pub description: Option<String>,
    pub default_task: Option<TaskOrCommand>,
    pub tasks: Option<std::collections::HashMap<String, TaskOrCommand>>,
}

/// A simplified union of "SimpleCommand" or a more advanced "ComplexTask"
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum TaskOrCommand {
    SimpleCommand {
        command: String,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        working_dir: Option<String>,
    },
    ComplexTask {
        #[serde(default)]
        command: Option<String>,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        working_dir: Option<String>,
    },
}

fn validate_yaml_for_duplicates(path: &Path) -> Result<(), PluginError> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        PluginError::GenericError(format!("Cannot read file {}: {}", path.display(), e))
    })?;

    // First, parse the content line by line to detect duplicate keys
    let mut default_count = 0;
    let mut task_names = std::collections::HashSet::new();
    let mut in_tasks = false;
    let mut tasks_indent = 0;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Calculate indentation level
        let indent = line.chars().take_while(|c| *c == ' ').count();

        // Check for default_task at root level
        if !in_tasks && trimmed.starts_with("default_task:") {
            default_count += 1;
            if default_count > 1 {
                return Err(PluginError::GenericError(format!(
                    "multiple default_task fields found in {}",
                    path.display()
                )));
            }
        }

        // Check for tasks section
        if trimmed == "tasks:" {
            in_tasks = true;
            tasks_indent = indent;
            continue;
        }

        // Inside tasks section, look for task names
        if in_tasks && indent == tasks_indent + 2 && trimmed.ends_with(':') {
            let task_name = trimmed.trim_end_matches(':').to_string();
            if !task_names.insert(task_name.clone()) {
                return Err(PluginError::GenericError(format!(
                    "duplicate task name '{}' found in {}",
                    task_name,
                    path.display()
                )));
            }
        }
    }

    Ok(())
}

/// Convert ScriptFile to Graph nodes.
impl ScriptFile {
    pub fn to_graph(&self, graph: &mut Graph) -> Result<(), PluginError> {
        // Add default task if present
        if let Some(default_task) = &self.default_task {
            match default_task {
                TaskOrCommand::SimpleCommand {
                    command,
                    description,
                    working_dir,
                } => {
                    let command_node = NodeKind::Command(CommandData {
                        raw_command: command.to_owned(),
                        description: description.clone(),
                        working_dir: working_dir.clone(),
                    });
                    graph.add_node(command_node);
                }
                TaskOrCommand::ComplexTask {
                    command,
                    description,
                    working_dir,
                } => {
                    if let Some(cmd) = command {
                        let command_node = NodeKind::Command(CommandData {
                            raw_command: cmd.to_owned(),
                            description: description.clone(),
                            working_dir: working_dir.clone(),
                        });
                        graph.add_node(command_node);
                    }
                }
            }
        }

        // Add named tasks
        if let Some(tasks) = &self.tasks {
            for (name, entry) in tasks {
                match entry {
                    TaskOrCommand::SimpleCommand {
                        command,
                        description,
                        working_dir,
                    } => {
                        // Create only a task node for each named task
                        let task_node = NodeKind::Task(TaskData {
                            name: name.clone(),
                            description: description.clone(),
                            command: Some(command.clone()),
                            working_dir: working_dir.clone(),
                        });
                        graph.add_node(task_node);
                    }
                    TaskOrCommand::ComplexTask {
                        command,
                        description,
                        working_dir,
                    } => {
                        // Create only a task node for each named task
                        let task_node = NodeKind::Task(TaskData {
                            name: name.clone(),
                            description: description.clone(),
                            command: command.clone(),
                            working_dir: working_dir.clone(),
                        });
                        graph.add_node(task_node);
                    }
                }
            }
        }

        Ok(())
    }
}

/// Utility to detect if the path is a glob
pub fn is_glob(p: &str) -> bool {
    p.contains('*') || p.contains('?') || (p.contains('[') && p.contains(']'))
}

/// Load bodo.toml or defaults
pub fn load_bodo_config<P: AsRef<Path>>(config_path: Option<P>) -> Result<BodoConfig, PluginError> {
    let path = config_path
        .as_ref()
        .map_or_else(|| PathBuf::from("bodo.toml"), |p| p.as_ref().to_path_buf());

    if path.exists() {
        let content = fs::read_to_string(&path)
            .map_err(|e| PluginError::GenericError(format!("Cannot read bodo.toml: {}", e)))?;
        let parsed: BodoConfig = toml::from_str(&content)
            .map_err(|e| PluginError::GenericError(format!("bodo.toml parse error: {}", e)))?;
        Ok(parsed)
    } else {
        // If no path was provided or default doesn't exist, use default config
        Ok(BodoConfig::default())
    }
}

/// Load scripts from the fs based on config
pub fn load_scripts_from_fs(config: &BodoConfig, graph: &mut Graph) -> Result<(), PluginError> {
    let paths_or_globs = config
        .script_paths
        .clone()
        .unwrap_or_else(|| vec!["scripts".to_string()]);

    println!("Searching in paths: {:?}", paths_or_globs);

    for pat in paths_or_globs {
        let path = PathBuf::from(&pat);
        println!("Processing path: {:?}", path);

        // If it's already a glob pattern, use it directly
        if is_glob(&pat) {
            println!("Found glob pattern: {}", pat);
            let glob_pattern = if !pat.contains("*.yaml") {
                format!("{}/**/*.yaml", pat.trim_end_matches('/'))
            } else {
                pat.to_string()
            };
            println!("Using glob pattern: {}", glob_pattern);
            process_glob_pattern(&glob_pattern, graph)?;
            continue;
        }

        // For everything else, try as a direct path first
        if path.is_file() && path.extension().is_some_and(|ext| ext == "yaml") {
            println!("Loading direct file: {:?}", path);
            load_single_yaml_file(&path, graph)?;
            continue;
        }

        // For directories or paths ending with /, use glob to find yaml files
        let base_path = if pat.ends_with('/') {
            pat.trim_end_matches('/').to_string()
        } else {
            pat
        };

        let glob_pattern = format!("{}/**/*.yaml", base_path);
        println!("Using glob pattern: {}", glob_pattern);
        process_glob_pattern(&glob_pattern, graph)?;
    }
    Ok(())
}

fn process_glob_pattern(pattern: &str, graph: &mut Graph) -> Result<(), PluginError> {
    // Convert relative path to absolute if needed
    let pattern = if Path::new(pattern).is_absolute() {
        pattern.to_string()
    } else {
        // For relative paths, just use them as is
        pattern.to_string()
    };

    // If the pattern doesn't exist and it's not a glob pattern, return Ok
    if !is_glob(&pattern) && !Path::new(&pattern).exists() {
        return Ok(());
    }

    for entry in glob(&pattern)
        .map_err(|e| PluginError::GenericError(format!("Bad glob pattern '{}': {}", pattern, e)))?
    {
        let path =
            entry.map_err(|e| PluginError::GenericError(format!("Failed glob entry: {}", e)))?;
        if path.is_file() {
            println!("Loading glob match: {:?}", path);
            load_single_yaml_file(&path, graph)?;
        }
    }
    Ok(())
}

fn load_single_yaml_file(path: &Path, graph: &mut Graph) -> Result<(), PluginError> {
    println!("Loading YAML file: {:?}", path);

    // Validate YAML structure first
    validate_yaml_for_duplicates(path)?;

    let content = fs::read_to_string(path).map_err(|e| {
        PluginError::GenericError(format!("File read error for {}: {}", path.display(), e))
    })?;

    let parsed: ScriptFile = serde_yaml::from_str(&content).map_err(|e| {
        PluginError::GenericError(format!("YAML parse error in {}: {}", path.display(), e))
    })?;

    println!("Parsed YAML: {:?}", parsed);
    parsed.to_graph(graph)?;
    Ok(())
}
