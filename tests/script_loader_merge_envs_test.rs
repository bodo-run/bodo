use bodo::script_loader::ScriptLoader;

#[test]
fn test_merge_envs_overrides() {
    let global = std::collections::HashMap::from([
        ("VAR".to_string(), "global".to_string()),
        ("A".to_string(), "1".to_string()),
    ]);
    let script = std::collections::HashMap::from([
        ("VAR".to_string(), "script".to_string()),
        ("B".to_string(), "2".to_string()),
    ]);
    let task = std::collections::HashMap::from([
        ("VAR".to_string(), "task".to_string()),
        ("C".to_string(), "3".to_string()),
    ]);
    let merged = ScriptLoader::merge_envs(&global, &script, &task);
    assert_eq!(merged.get("VAR"), Some(&"task".to_string()));
    assert_eq!(merged.get("A"), Some(&"1".to_string()));
    assert_eq!(merged.get("B"), Some(&"2".to_string()));
    assert_eq!(merged.get("C"), Some(&"3".to_string()));
}
