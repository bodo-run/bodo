use bodo::cli::Args;
use bodo::plugin::PluginConfig;
use bodo::plugins::execution_plugin::ExecutionPlugin;
use bodo::plugin::Plugin;
use bodo::graph::Graph;
use clap::Parser;

#[test]
fn test_dry_run_flag_parsing() {
    let args = Args::parse_from(["bodo", "--dry-run", "test_task"]);
    assert!(args.dry_run);
    assert_eq!(args.task, Some("test_task".to_string()));
}

#[test]
fn test_execution_plugin_dry_run_mode() {
    let mut plugin = ExecutionPlugin::new();
    
    // Test plugin configuration with dry_run = true
    let config = PluginConfig {
        fail_fast: true,
        watch: false,
        list: false,
        dry_run: true,
        options: Some({
            let mut options = serde_json::Map::new();
            options.insert("task".to_string(), serde_json::Value::String("test_task".to_string()));
            options
        }),
    };
    
    plugin.on_init(&config).expect("Plugin initialization should succeed");
    assert!(plugin.dry_run, "Plugin should be in dry-run mode");
    assert_eq!(plugin.task_name, Some("test_task".to_string()));
}

#[test]
fn test_execution_plugin_normal_mode() {
    let mut plugin = ExecutionPlugin::new();
    
    // Test plugin configuration with dry_run = false
    let config = PluginConfig {
        fail_fast: true,
        watch: false,
        list: false,
        dry_run: false,
        options: Some({
            let mut options = serde_json::Map::new();
            options.insert("task".to_string(), serde_json::Value::String("test_task".to_string()));
            options
        }),
    };
    
    plugin.on_init(&config).expect("Plugin initialization should succeed");
    assert!(!plugin.dry_run, "Plugin should not be in dry-run mode");
    assert_eq!(plugin.task_name, Some("test_task".to_string()));
}

#[test]
fn test_dry_run_with_empty_graph() {
    let mut plugin = ExecutionPlugin::new();
    plugin.dry_run = true;
    plugin.task_name = Some("nonexistent_task".to_string());
    
    let mut graph = Graph::new();
    // This should return an error because the task doesn't exist
    let result = plugin.on_after_run(&mut graph);
    assert!(result.is_err());
}