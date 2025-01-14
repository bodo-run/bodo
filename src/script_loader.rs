use std::{
    fs,
    path::{Path, PathBuf},
};

use glob::glob;
use serde::Deserialize;
use toml;
use walkdir::WalkDir;

use crate::{
    errors::PluginError,
    graph::{Graph, NodeKind},
};

#[derive(Debug, Deserialize)]
pub struct BodoConfig {
    pub script_paths: Option<Vec<String>>,
}

impl Default for BodoConfig {
    fn default() -> Self {
        BodoConfig { script_paths: None }
    }
}

#[derive(Debug, Deserialize)]
pub struct ScriptFile {
    pub name: Option<String>,
    pub description: Option<String>,
    pub default_task: Option<TaskOrCommand>,
    pub tasks: Option<std::collections::HashMap<String, TaskOrCommand>>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum TaskOrCommand {
    SimpleCommand {
        command: String,
        #[serde(default)]
        description: Option<String>,
    },
    ComplexTask {
        concurrently: Option<Vec<TaskOrCommand>>,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        command: Option<String>,
    },
}

impl ScriptFile {
    pub fn to_graph(&self, graph: &mut Graph) -> Result<(), PluginError> {
        if let Some(def_task) = &self.default_task {
            match def_task {
                TaskOrCommand::SimpleCommand {
                    command,
                    description,
                } => {
                    let nodekind = NodeKind::Command(crate::graph::CommandData {
                        raw_command: command.to_owned(),
                        description: description.clone(),
                    });
                    graph.add_node(nodekind);
                }
                TaskOrCommand::ComplexTask {
                    command,
                    description,
                    ..
                } => {
                    if let Some(raw_cmd) = command {
                        let nodekind = NodeKind::Command(crate::graph::CommandData {
                            raw_command: raw_cmd.to_owned(),
                            description: description.clone(),
                        });
                        graph.add_node(nodekind);
                    }
                }
            }
        }

        if let Some(tasks_map) = &self.tasks {
            for (name, task_data) in tasks_map {
                match task_data {
                    TaskOrCommand::SimpleCommand {
                        command,
                        description,
                    } => {
                        let nodekind = NodeKind::Task(crate::graph::TaskData {
                            name: name.to_string(),
                            description: description.clone(),
                        });
                        let node_id = graph.add_node(nodekind);
                        let node = graph
                            .nodes
                            .iter_mut()
                            .find(|n| n.id == node_id)
                            .expect("Node we just added must exist");
                        node.metadata.insert("command".to_string(), command.clone());
                    }
                    TaskOrCommand::ComplexTask {
                        command,
                        description,
                        concurrently: _,
                        ..
                    } => {
                        let nodekind = NodeKind::Task(crate::graph::TaskData {
                            name: name.to_string(),
                            description: description.clone(),
                        });
                        let node_id = graph.add_node(nodekind);
                        if let Some(cmd) = command {
                            let node = graph
                                .nodes
                                .iter_mut()
                                .find(|n| n.id == node_id)
                                .expect("Node we just added must exist");
                            node.metadata.insert("command".to_string(), cmd.clone());
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

pub fn is_glob(s: &str) -> bool {
    s.contains('*') || s.contains('?') || (s.contains('[') && s.contains(']'))
}

pub fn load_bodo_config<P: AsRef<Path>>(config_path: Option<P>) -> Result<BodoConfig, PluginError> {
    let path = config_path
        .as_ref()
        .map_or_else(|| PathBuf::from("bodo.toml"), |p| p.as_ref().to_path_buf());

    if path.exists() {
        let content = fs::read_to_string(&path)
            .map_err(|e| PluginError::GenericError(format!("Cannot read bodo.toml: {}", e)))?;
        let parsed_config: BodoConfig = toml::from_str(&content)
            .map_err(|e| PluginError::GenericError(format!("bodo.toml parse error: {}", e)))?;
        Ok(parsed_config)
    } else {
        Ok(BodoConfig::default())
    }
}

pub fn load_scripts_from_fs(config: &BodoConfig, graph: &mut Graph) -> Result<(), PluginError> {
    let paths_or_globs = config
        .script_paths
        .clone()
        .unwrap_or_else(|| vec![String::from("scripts/")]);

    for pat in paths_or_globs {
        if is_glob(&pat) {
            for entry in glob(&pat)
                .map_err(|e| PluginError::GenericError(format!("Bad glob pattern: {}", e)))?
            {
                let path = entry.map_err(|e| {
                    PluginError::GenericError(format!("Failed to process glob entry: {}", e))
                })?;
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

fn load_single_yaml_file(path: &Path, graph: &mut Graph) -> Result<(), PluginError> {
    if path.extension().map_or(false, |ext| ext == "yaml") {
        let script_def = parse_script_file(path)?;
        script_def.to_graph(graph)?;
    }
    Ok(())
}

fn load_yaml_files_in_dir(dir_path: &Path, graph: &mut Graph) -> Result<(), PluginError> {
    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "yaml") {
            let script_def = parse_script_file(path)?;
            script_def.to_graph(graph)?;
        }
    }
    Ok(())
}

fn parse_script_file(path: &Path) -> Result<ScriptFile, PluginError> {
    let content = fs::read_to_string(path)
        .map_err(|e| PluginError::GenericError(format!("File read error: {}", e)))?;
    let parsed: ScriptFile = serde_yaml::from_str(&content)
        .map_err(|e| PluginError::GenericError(format!("YAML parse error: {}", e)))?;
    Ok(parsed)
}
