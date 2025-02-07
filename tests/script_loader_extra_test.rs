use crate::script_loader::ScriptLoader;
use std::collections::HashMap;

#[test]
fn test_merge_envs() {
    let global = HashMap::from([
        ("A".to_string(), "1".to_string()),
        ("SHARED".to_string(), "global".to_string()),
    ]);
    let script = HashMap::from([
        ("B".to_string(), "2".to_string()),
        ("SHARED".to_string(), "script".to_string()),
    ]);
    let task = HashMap::from([
        ("C".to_string(), "3".to_string()),
        ("SHARED".to_string(), "task".to_string()),
    ]);
    let merged = ScriptLoader::merge_envs(&global, &script, &task);
    assert_eq!(merged.get("A"), Some(&"1".to_string()));
    assert_eq!(merged.get("B"), Some(&"2".to_string()));
    assert_eq!(merged.get("C"), Some(&"3".to_string()));
    assert_eq!(merged.get("SHARED"), Some(&"task".to_string())); // Task overrides script and global
}

#[test]
fn test_merge_exec_paths() {
    // Test basic merging
    let global = vec!["/global".to_string()];
    let script = vec!["/script".to_string()];
    let task = vec!["/task".to_string()];
    let merged = ScriptLoader::merge_exec_paths(&global, &script, &task);
    assert_eq!(
        merged,
        vec![
            "/global".to_string(),
            "/script".to_string(),
            "/task".to_string()
        ]
    );

    // Test empty vectors
    let empty: Vec<String> = vec![];
    let merged_empty = ScriptLoader::merge_exec_paths(&empty, &empty, &empty);
    assert!(merged_empty.is_empty());

    // Test duplicate paths
    let global_dup = vec!["/shared".to_string()];
    let script_dup = vec!["/shared".to_string()];
    let task_dup = vec!["/shared".to_string()];
    let merged_dup = ScriptLoader::merge_exec_paths(&global_dup, &script_dup, &task_dup);
    assert_eq!(merged_dup, vec!["/shared".to_string()]);
}
