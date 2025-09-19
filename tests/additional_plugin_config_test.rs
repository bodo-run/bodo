use bodo::plugin::PluginConfig;
use serde_json::json;

#[test]
fn test_plugin_config_defaults() {
    let config = PluginConfig::default();
    assert!(!config.fail_fast);
    assert!(!config.watch);
    assert!(!config.list);
    assert!(config.options.is_none());
}

#[test]
fn test_plugin_config_custom() {
    let options = json!({
        "task": "example"
    });
    let config = PluginConfig {
        fail_fast: true,
        watch: true,
        list: true,
        dry_run: false,
        enable_recovery: false,
        max_retry_attempts: None,
        initial_retry_backoff: None,
        options: Some(options.as_object().unwrap().clone()),
    };
    assert!(config.fail_fast);
    assert!(config.watch);
    assert!(config.list);
    let opts = config.options.unwrap();
    assert_eq!(opts.get("task").unwrap(), "example");
}
