use bodo::config::WatchConfig;
use bodo::graph::{NodeKind, TaskData};
use bodo::plugin::Plugin;
use bodo::plugins::watch_plugin::WatchPlugin;
use bodo::Graph;
use std::collections::HashMap;

#[test]
fn test_watch_plugin_on_graph_build_with_tasks() {
    let mut plugin = WatchPlugin::new(true, false);
    let mut graph = Graph::new();

    let task_data = TaskData {
        name: "watch_task".to_string(),
        description: None,
        command: Some("echo 'Watching files'".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "script".to_string(),
        script_display_name: "script".to_string(),
        watch: Some(WatchConfig {
            patterns: vec!["src/**/*.rs".to_string()],
            debounce_ms: 500,
            ignore_patterns: vec![],
            auto_watch: true,
        }),
    };

    let _node_id = graph.add_node(NodeKind::Task(task_data)); // Prefixed `node_id` with an underscore to avoid unused variable warning
    plugin.on_graph_build(&mut graph).unwrap();

    // Ensure that the watch_entries were populated
    assert_eq!(plugin.get_watch_entry_count(), 1);

    // Since we cannot access private fields, we cannot verify internal data
}
