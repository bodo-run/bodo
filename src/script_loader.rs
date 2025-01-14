use std::{
    fs,
    path::{Path, PathBuf},
};

use glob::glob;
use serde::Deserialize;
use walkdir::WalkDir;

use crate::{
    errors::PluginError,
    graph::{CommandData, Graph, NodeKind, TaskData},
};

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
    },
    ComplexTask {
        #[serde(default)]
        command: Option<String>,
        #[serde(default)]
        description: Option<String>,
    },
}

/// Convert ScriptFile to Graph nodes.
impl ScriptFile {
    pub fn to_graph(&self, graph: &mut Graph) -> Result<(), PluginError> {
        // If default_task is present, interpret as a command node
        if let Some(def_task) = &self.default_task {
            match def_task {
                TaskOrCommand::SimpleCommand {
                    command,
                    description,
                } => {
                    let node_kind = NodeKind::Command(CommandData {
                        raw_command: command.to_owned(),
                        description: description.clone(),
                    });
                    graph.add_node(node_kind);
                }
                TaskOrCommand::ComplexTask {
                    command,
                    description,
                } => {
                    if let Some(cmd_str) = command {
                        let node_kind = NodeKind::Command(CommandData {
                            raw_command: cmd_str.to_owned(),
                            description: description.clone(),
                        });
                        graph.add_node(node_kind);
                    }
                }
            }
        }

        // Then handle named tasks
        if let Some(tasks_map) = &self.tasks {
            for (name, entry) in tasks_map {
                match entry {
                    TaskOrCommand::SimpleCommand {
                        command: _,
                        description,
                    } => {
                        let node_kind = NodeKind::Task(TaskData {
                            name: name.clone(),
                            description: description.clone(),
                        });
                        graph.add_node(node_kind);
                    }
                    TaskOrCommand::ComplexTask {
                        command: _,
                        description,
                    } => {
                        let node_kind = NodeKind::Task(TaskData {
                            name: name.clone(),
                            description: description.clone(),
                        });
                        graph.add_node(node_kind);
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
        Ok(BodoConfig::default())
    }
}

/// Load scripts from the fs based on config
pub fn load_scripts_from_fs(config: &BodoConfig, graph: &mut Graph) -> Result<(), PluginError> {
    let paths_or_globs = config
        .script_paths
        .clone()
        .unwrap_or_else(|| vec!["scripts/".to_string()]);

    for pat in paths_or_globs {
        if is_glob(&pat) {
            for entry in glob(&pat)
                .map_err(|e| PluginError::GenericError(format!("Bad glob pattern: {}", e)))?
            {
                let path = entry
                    .map_err(|e| PluginError::GenericError(format!("Failed glob entry: {}", e)))?;
                if path.is_dir() {
                    load_yaml_files_in_dir(&path, graph)?;
                } else {
                    load_single_yaml_file(&path, graph)?;
                }
            }
        } else {
            let path = PathBuf::from(&pat);
            if path.is_dir() {
                load_yaml_files_in_dir(&path, graph)?;
            } else if path.is_file() {
                load_single_yaml_file(&path, graph)?;
            }
        }
    }
    Ok(())
}

fn load_yaml_files_in_dir(dir_path: &Path, graph: &mut Graph) -> Result<(), PluginError> {
    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "yaml") {
            load_single_yaml_file(path, graph)?;
        }
    }
    Ok(())
}

fn load_single_yaml_file(path: &Path, graph: &mut Graph) -> Result<(), PluginError> {
    let content = fs::read_to_string(path)
        .map_err(|e| PluginError::GenericError(format!("File read error: {}", e)))?;
    let parsed: ScriptFile = serde_yaml::from_str(&content)
        .map_err(|e| PluginError::GenericError(format!("YAML parse error: {}", e)))?;
    parsed.to_graph(graph)?;
    Ok(())
}
