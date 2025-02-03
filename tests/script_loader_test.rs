use bodo::config::{BodoConfig, Dependency, TaskConfig};
use bodo::errors::{BodoError, Result};
use bodo::script_loader::ScriptLoader;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[test]
fn test_build_graph_empty_config() {
    let config = BodoConfig::default();
    let mut loader = ScriptLoader::new();
    let result = loader.build_graph(config);
    assert!(result.is_ok());
}

#[test]
fn test_load_nonexistent_script() {
    let mut loader = ScriptLoader::new();
    let mut _graph = bodo::graph::Graph::new();
    let path = PathBuf::from("nonexistent_script.yaml");
    let result = loader.load_script(&mut _graph, &path, "", &HashMap::new(), &[]);
    assert!(result.is_err());
}

#[test]
fn test_merge_envs() {
    let global_env = HashMap::from([("GLOBAL".to_string(), "1".to_string())]);
    let script_env = HashMap::from([("SCRIPT".to_string(), "2".to_string())]);
    let task_env = HashMap::from([("TASK".to_string(), "3".to_string())]);
    let merged = ScriptLoader::merge_envs(&global_env, &script_env, &task_env);
    assert_eq!(merged.get("GLOBAL"), Some(&"1".to_string()));
    assert_eq!(merged.get("SCRIPT"), Some(&"2".to_string()));
    assert_eq!(merged.get("TASK"), Some(&"3".to_string()));
}

#[test]
fn test_register_duplicate_task() {
    let mut loader = ScriptLoader::new();
    let mut graph = bodo::graph::Graph::new();
    let config = BodoConfig {
        tasks: HashMap::from([
            ("task1".to_string(), Default::default()),
            ("task1".to_string(), Default::default()),
        ]),
        ..Default::default()
    };
    let result = loader.build_graph(config);
    assert!(result.is_err());
    if let Err(BodoError::PluginError(msg)) = result {
        assert!(msg.contains("Duplicate task name"));
    } else {
        panic!("Expected PluginError due to duplicate task name");
    }
}

#[test]
fn test_resolve_dependency_not_found() {
    let mut loader = ScriptLoader::new();
    let config = BodoConfig {
        tasks: HashMap::from([(
            "task1".to_string(),
            TaskConfig {
                pre_deps: vec![Dependency::Task {
                    task: "nonexistent".to_string(),
                }],
                ..Default::default()
            },
        )]),
        ..Default::default()
    };

    let result = loader.build_graph(config);
    assert!(result.is_err());
    if let Err(BodoError::PluginError(msg)) = result {
        assert!(msg.contains("Dependency not found"));
    } else {
        panic!("Expected PluginError due to missing dependency");
    }
}

#[test]
fn test_parse_cross_file_ref() {
    let loader = ScriptLoader::new();
    let referencing_file = PathBuf::from("dir/script.yaml");
    let dep = "../other.yaml/some-task";
    let result = loader.parse_cross_file_ref(dep, &referencing_file);
    assert!(result.is_some());
    let (script_path, task_name) = result.unwrap();
    assert_eq!(script_path, PathBuf::from("dir/../other.yaml"));
    assert_eq!(task_name, "some-task".to_string());
}

#[test]
fn test_parse_cross_file_ref_no_slash() {
    let loader = ScriptLoader::new();
    let referencing_file = PathBuf::from("dir/script.yaml");
    let dep = "task-name";
    let result = loader.parse_cross_file_ref(dep, &referencing_file);
    assert!(result.is_none());
}
