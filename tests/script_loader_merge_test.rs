use bodo::script_loader::ScriptLoader;
use std::collections::HashMap;

#[test]
fn test_merge_envs_override_order() {
    let global = HashMap::from([("KEY".to_string(), "global".to_string())]);
    let script = HashMap::from([("KEY".to_string(), "script".to_string())]);
    let task = HashMap::from([("KEY".to_string(), "task".to_string())]);
    let merged = ScriptLoader::merge_envs(&global, &script, &task);
    // Task should override script and global.
    assert_eq!(merged.get("KEY"), Some(&"task".to_string()));
}

#[test]
fn test_merge_exec_paths_no_duplicates() {
    let global = vec!["/path1".to_string(), "/path2".to_string()];
    let script = vec!["/path2".to_string(), "/path3".to_string()];
    let task = vec!["/path1".to_string(), "/path4".to_string()];
    let merged = ScriptLoader::merge_exec_paths(&global, &script, &task);
    let expected = vec![
        "/path1".to_string(),
        "/path2".to_string(),
        "/path3".to_string(),
        "/path4".to_string(),
    ];
    assert_eq!(merged, expected);
}
