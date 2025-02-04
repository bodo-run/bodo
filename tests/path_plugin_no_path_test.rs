use bodo::plugins::path_plugin::PathPlugin;
use std::env;

#[test]
fn test_build_path_no_path_variable() {
    let mut plugin = PathPlugin::new();
    plugin.set_default_paths(vec!["/default".to_string()]);
    plugin.set_preserve_path(true);
    // Remove PATH variable temporarily
    env::remove_var("PATH");
    let result = plugin.test_build_path(None, &["/exec".to_string()]);
    // Expected: join default_paths and exec_paths only, as PATH is not available
    assert_eq!(result, "/default:/exec");
}
