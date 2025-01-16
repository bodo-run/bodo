use bodo::{
    graph::{Graph, NodeKind, TaskData},
    plugin::{Plugin, PluginConfig},
    plugins::print_list_plugin::PrintListPlugin,
};

#[test]
fn test_print_list_plugin_shows_help() {
    // Build a small graph with 2 tasks (root-level, plus a script-sourced one)
    let mut graph = Graph::new();

    // Root level task
    let root_id = graph.add_node(NodeKind::Task(TaskData {
        name: "build".to_string(),
        description: Some("Build something".to_string()),
        command: None,
        working_dir: None,
        is_default: false,
        script_name: Some("Root".to_string()),
    }));

    // Another script-sourced
    let script_id = graph.add_node(NodeKind::Task(TaskData {
        name: "clippy".to_string(),
        description: Some("Run clippy".to_string()),
        command: None,
        working_dir: None,
        is_default: false,
        script_name: Some("code_quality".to_string()),
    }));

    // Plugin with show_help = true
    let mut plugin = PrintListPlugin::new(true);

    // Run the plugin - this will print to stdout
    futures::executor::block_on(async {
        // init
        plugin.on_init(&PluginConfig::default()).await.unwrap();
        // run on_graph_build
        plugin.on_graph_build(&mut graph).await.unwrap();
    });

    // Note: We can't verify the exact output since we're not capturing stdout,
    // but we can at least verify the plugin runs without errors
}
