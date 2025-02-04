use bodo::script_loader::ScriptLoader;

#[test]
fn test_merge_exec_paths_no_duplicates_order() {
    let global = vec!["/a".to_string(), "/b".to_string()];
    let script = vec!["/b".to_string(), "/c".to_string()];
    let task = vec!["/a".to_string(), "/d".to_string()];
    let merged = ScriptLoader::merge_exec_paths(&global, &script, &task);
    assert_eq!(
        merged,
        vec![
            "/a".to_string(),
            "/b".to_string(),
            "/c".to_string(),
            "/d".to_string()
        ]
    );
}
