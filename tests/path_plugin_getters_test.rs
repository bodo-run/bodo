extern crate bodo;
use bodo::plugins::path_plugin::PathPlugin;

#[test]
fn test_getters_setters() {
    let mut plugin = PathPlugin::new();
    // Initially default paths is empty and preserve_path is true.
    assert!(plugin.get_default_paths().is_empty());
    plugin.set_default_paths(vec!["/default".to_string()]);
    assert_eq!(plugin.get_default_paths().len(), 1);
    plugin.set_preserve_path(false);
    assert!(!plugin.get_preserve_path());
}
