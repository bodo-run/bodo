use bodo::plugins::path_plugin::PathPlugin;
use std::env;

#[test]
fn test_build_path_with_working_dir_and_no_preserve() {
    let mut plugin = PathPlugin::new();
    plugin.set_default_paths(vec!["/default".to_string()]);
    plugin.set_preserve_path(false);
    let work_dir = "/work".to_string();
    let working_dir = Some(&work_dir);
    let exec_paths = vec!["/exec".to_string()];
    let result = plugin.test_build_path(working_dir, &exec_paths);
    assert_eq!(result, "/work:/default:/exec");
}

#[test]
fn test_build_path_with_no_working_dir_and_preserve() {
    let mut plugin = PathPlugin::new();
    plugin.set_default_paths(vec!["/default".to_string()]);
    plugin.set_preserve_path(true);
    let original_path = env::var("PATH");
    env::set_var("PATH", "/existing");
    // Test code here
    if let Ok(path) = original_path {
        env::set_var("PATH", path);
    } else {
        env::remove_var("PATH");
    }
    let working_dir = None;
    let exec_paths = vec!["/exec".to_string()];
    let result = plugin.test_build_path(working_dir, &exec_paths);
    // Expected order: default_paths then exec_paths then existing PATH (since working_dir is None)
    assert_eq!(result, "/default:/exec:/existing");
}
