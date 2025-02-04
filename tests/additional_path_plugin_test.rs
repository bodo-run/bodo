use bodo::plugins::path_plugin::PathPlugin;

#[test]
fn test_path_plugin_test_build_path_with_working_dir() {
    let mut plugin = PathPlugin::new();
    plugin.set_default_paths(vec!["/default".to_string()]);
    plugin.set_preserve_path(false);
    let working_dir = Some(&"/work".to_string());
    let exec_paths = vec!["/exec".to_string()];
    let result = plugin.test_build_path(working_dir, &exec_paths);
    // Expected: working_dir, then default, then exec_paths.
    assert_eq!(result, "/work:/default:/exec");
}

#[test]
fn test_path_plugin_test_build_path_without_working_dir() {
    let mut plugin = PathPlugin::new();
    plugin.set_default_paths(vec!["/default".to_string()]);
    plugin.set_preserve_path(false);
    let working_dir = None;
    let exec_paths = vec!["/exec".to_string()];
    let result = plugin.test_build_path(working_dir, &exec_paths);
    // Expected: default_paths then exec_paths.
    assert_eq!(result, "/default:/exec");
}
