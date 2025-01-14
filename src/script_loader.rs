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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir_all, write};
    use tempfile::tempdir;

    // BodoConfig Tests
    #[test]
    fn test_default_config_when_no_bodo_toml() {
        let config = load_bodo_config::<&str>(None).unwrap();
        assert!(
            config.script_paths.is_none(),
            "Expected None for script_paths by default"
        );
    }

    #[test]
    fn test_load_valid_toml_config() {
        let temp = tempdir().unwrap();
        let config_path = temp.path().join("bodo.toml");

        let toml_content = r#"
script_paths = ["my-scripts/", "others/*.yaml"]
        "#;

        write(&config_path, toml_content).unwrap();

        let loaded = load_bodo_config(Some(config_path)).unwrap();

        assert_eq!(
            loaded.script_paths,
            Some(vec!["my-scripts/".to_string(), "others/*.yaml".to_string()])
        );
    }

    #[test]
    fn test_load_invalid_toml_config() {
        let temp = tempdir().unwrap();
        let config_path = temp.path().join("bodo.toml");

        let bad_toml = r#"
script_paths = ["scripts/]
"#;

        write(&config_path, bad_toml).unwrap();

        let result = load_bodo_config(Some(&config_path));
        match result {
            Err(PluginError::GenericError(msg)) => {
                assert!(
                    msg.contains("bodo.toml parse error"),
                    "Should mention a TOML parse error"
                );
            }
            _ => panic!("Expected GenericError for invalid TOML"),
        }
    }

    #[test]
    fn test_file_missing_read_permission() {
        let temp = tempdir().unwrap();
        let config_path = temp.path().join("bodo.toml");

        write(&config_path, "script_paths = [\"scripts/\"]").unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&config_path).unwrap().permissions();
            perms.set_mode(0o200); // Write-only
            fs::set_permissions(&config_path, perms).unwrap();

            let result = load_bodo_config(Some(&config_path));
            match result {
                Err(PluginError::GenericError(msg)) => {
                    assert!(msg.contains("Cannot read bodo.toml"), "Expected read error");
                }
                _ => panic!("Expected error for unreadable file"),
            }
        }
    }

    #[test]
    fn test_unknown_fields_in_toml_are_ignored() {
        let temp = tempdir().unwrap();
        let config_path = temp.path().join("bodo.toml");

        let extended_toml = r#"
script_paths = ["scripts/"]
some_extra_field = "Whatever"
another_one = 123
"#;
        write(&config_path, extended_toml).unwrap();

        let loaded = load_bodo_config(Some(&config_path)).unwrap();
        assert_eq!(loaded.script_paths, Some(vec!["scripts/".to_string()]));
    }

    #[test]
    fn test_specify_config_path_non_existent() {
        let result = load_bodo_config(Some("nonexistent/bodo.toml"));
        let config = result.unwrap();
        assert!(config.script_paths.is_none());
    }

    #[test]
    fn test_load_single_yaml_file() {
        let temp = tempdir().unwrap();
        let script_path = temp.path().join("single.yaml");

        let yaml_content = r#"
name: "Single Test"
description: "Testing single-file load"

default_task:
  command: "echo default"
  description: "My default command"

tasks:
  build:
    command: "cargo build"
    description: "Build the project"
"#;

        write(&script_path, yaml_content).unwrap();

        let config = BodoConfig {
            script_paths: Some(vec![script_path.to_string_lossy().into_owned()]),
        };

        let mut graph = Graph::new();
        load_scripts_from_fs(&config, &mut graph).unwrap();

        assert_eq!(graph.nodes.len(), 2);

        match &graph.nodes[0].kind {
            NodeKind::Command(cmd) => {
                assert_eq!(cmd.raw_command, "echo default");
                assert_eq!(cmd.description.as_deref(), Some("My default command"));
            }
            _ => panic!("Expected Command node for default_task"),
        }

        match &graph.nodes[1].kind {
            NodeKind::Task(task) => {
                assert_eq!(task.name, "build");
                assert_eq!(task.description.as_deref(), Some("Build the project"));
            }
            _ => panic!("Expected Task node for 'build'"),
        }
    }

    #[test]
    fn test_load_multiple_files_in_directory() {
        let temp = tempdir().unwrap();
        let scripts_dir = temp.path().join("scripts");
        create_dir_all(&scripts_dir).unwrap();

        let script_a = scripts_dir.join("scriptA.yaml");
        let yaml_a = r#"
default_task:
  command: "echo from A"
tasks:
  foo:
    command: "echo Foo"
"#;
        write(&script_a, yaml_a).unwrap();

        let script_b = scripts_dir.join("scriptB.yaml");
        let yaml_b = r#"
default_task:
  command: "echo from B"
tasks:
  bar:
    command: "echo Bar"
"#;
        write(&script_b, yaml_b).unwrap();

        let config = BodoConfig {
            script_paths: Some(vec![scripts_dir.to_string_lossy().into_owned()]),
        };

        let mut graph = Graph::new();
        load_scripts_from_fs(&config, &mut graph).unwrap();

        assert_eq!(graph.nodes.len(), 4);

        let commands = graph
            .nodes
            .iter()
            .filter(|n| matches!(n.kind, NodeKind::Command(_)));
        let tasks = graph
            .nodes
            .iter()
            .filter(|n| matches!(n.kind, NodeKind::Task(_)));

        assert_eq!(commands.count(), 2);
        assert_eq!(tasks.count(), 2);
    }

    #[test]
    fn test_load_with_glob_pattern() {
        let temp = tempdir().unwrap();
        let scripts_dir = temp.path().join("some_dir");
        create_dir_all(&scripts_dir).unwrap();

        let file1 = scripts_dir.join("script1.yaml");
        let file2 = scripts_dir.join("script2.yaml");

        write(&file1, "default_task:\n  command: \"echo 111\"").unwrap();
        write(&file2, "default_task:\n  command: \"echo 222\"").unwrap();

        let pattern = format!("{}/**/*.yaml", scripts_dir.display());

        let config = BodoConfig {
            script_paths: Some(vec![pattern]),
        };

        let mut graph = Graph::new();
        load_scripts_from_fs(&config, &mut graph).unwrap();

        assert_eq!(graph.nodes.len(), 2);
        assert!(graph
            .nodes
            .iter()
            .all(|n| matches!(n.kind, NodeKind::Command(_))));
    }

    #[test]
    fn test_invalid_yaml() {
        let temp = tempdir().unwrap();
        let script_path = temp.path().join("invalid.yaml");

        // This is invalid YAML with mismatched quotes and invalid structure
        let bad_yaml = r#"
default_task: {
  command: "echo BAD
  description: 'unclosed quote
  invalid: [1, 2,
"#;

        write(&script_path, bad_yaml).unwrap();

        let config = BodoConfig {
            script_paths: Some(vec![script_path.to_string_lossy().into_owned()]),
        };
        let mut graph = Graph::new();

        let result = load_scripts_from_fs(&config, &mut graph);

        match result {
            Err(PluginError::GenericError(msg)) => {
                assert!(
                    msg.contains("YAML parse error"),
                    "Should mention parse error"
                );
            }
            _ => panic!("Expected a GenericError due to invalid YAML"),
        }
    }

    #[test]
    fn test_non_existent_path() {
        let config = BodoConfig {
            script_paths: Some(vec!["this/path/does/not/exist".to_string()]),
        };
        let mut graph = Graph::new();

        let result = load_scripts_from_fs(&config, &mut graph);
        assert!(
            result.is_ok(),
            "We skip non-existent directories by default"
        );
        assert_eq!(graph.nodes.len(), 0);
    }

    #[test]
    fn test_empty_scripts() {
        let temp = tempdir().unwrap();
        let empty_dir = temp.path().join("scripts_empty");
        create_dir_all(&empty_dir).unwrap();

        let config = BodoConfig {
            script_paths: Some(vec![empty_dir.to_string_lossy().into_owned()]),
        };
        let mut graph = Graph::new();
        load_scripts_from_fs(&config, &mut graph).unwrap();

        assert_eq!(graph.nodes.len(), 0);
    }

    #[test]
    fn test_complex_task_unused_fields() {
        let temp = tempdir().unwrap();
        let script_path = temp.path().join("complex.yaml");

        let complex_yaml = r#"
default_task:
  command: "./do_something.sh"
  description: "Complex default"
  concurrently:
    - command: "echo one"
    - command: "echo two"
tasks:
  alpha:
    command: "echo ALPHA"
"#;

        write(&script_path, complex_yaml).unwrap();

        let config = BodoConfig {
            script_paths: Some(vec![script_path.to_string_lossy().into_owned()]),
        };
        let mut graph = Graph::new();
        load_scripts_from_fs(&config, &mut graph).unwrap();

        assert_eq!(graph.nodes.len(), 2);

        match &graph.nodes[0].kind {
            NodeKind::Command(cmd) => {
                assert_eq!(cmd.raw_command, "./do_something.sh");
                assert_eq!(cmd.description.as_deref(), Some("Complex default"));
            }
            _ => panic!("Expected Command node"),
        }

        match &graph.nodes[1].kind {
            NodeKind::Task(td) => {
                assert_eq!(td.name, "alpha");
            }
            _ => panic!("Expected Task node named 'alpha'"),
        }
    }
}
