extern crate bodo;
use bodo::script_loader::ScriptLoader;

#[test]
fn test_merge_empty() {
    // Test that merging empty global, script, and task exec_paths returns an empty vector.
    let global: Vec<String> = vec![];
    let script: Vec<String> = vec![];
    let task: Vec<String> = vec![];
    let merged = ScriptLoader::merge_exec_paths(&global, &script, &task);
    assert!(merged.is_empty());
}
