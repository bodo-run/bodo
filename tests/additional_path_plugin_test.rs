use bodo::plugins::path_plugin::PathPlugin;

#[test]
fn test_build_path_with_working_dir_and_no_preserve() {
    let mut plugin = PathPlugin::new();
    plugin.set_default_paths(vec!["/default".to_string()]);
    plugin.set_preserve_path(false);
    let work = "/work".to_string();
    let working_dir = Some(&work);
    let exec_paths = vec!["/exec".to_string()];
    let result = plugin.test_build_path(working_dir, &exec_paths);
    assert_eq!(result, "/work:/default:/exec");
}

#[test]
fn test_build_path_with_no_working_dir_and_preserve() {
    let mut plugin = PathPlugin::new();
    plugin.set_default_paths(vec!["/default".to_string()]);
    plugin.set_preserve_path(true);
    let original_path = std::env::var("PATH").ok();
    std::env::set_var("PATH", "/existing");
    let working_dir = None;
    let exec_paths = vec!["/exec".to_string()];
    let result = plugin.test_build_path(working_dir, &exec_paths);
    // Restore the original PATH after calling test_build_path
    if let Some(path) = original_path {
        std::env::set_var("PATH", path);
    } else {
        std::env::remove_var("PATH");
    }
    // Expected order: default_paths then exec_paths then existing PATH (since working_dir is None)
    assert_eq!(result, "/default:/exec:/existing");
}
