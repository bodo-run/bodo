use bodo::config::BodoConfig;
use bodo::script_loader::ScriptLoader;
use std::collections::HashMap;
use std::path::PathBuf;

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
    let mut graph = bodo::graph::Graph::new();
    let path = PathBuf::from("nonexistent_script.yaml");
    let result = loader.load_script(&mut graph, &path, "", &HashMap::new(), &[]);
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
