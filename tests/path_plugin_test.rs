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
    let original = env::var("PATH").unwrap_or_default();
    env::set_var("PATH", "/existing");
    let working_dir = None;
    let exec_paths = vec!["/exec".to_string()];
    let result = plugin.test_build_path(working_dir, &exec_paths);
    env::set_var("PATH", original);
    assert_eq!(result, "/default:/exec:/existing");
}
