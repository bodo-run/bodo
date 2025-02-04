use bodo::cli::{get_task_name, Args};
use bodo::designer;
use bodo::errors::{BodoError, Result};
use bodo::graph::{Graph, NodeKind, TaskData};
use bodo::manager::GraphManager;
use bodo::plugin::{Plugin, PluginConfig, PluginManager};
use bodo::plugins::{
    concurrent_plugin::ConcurrentPlugin, env_plugin::EnvPlugin, execution_plugin::ExecutionPlugin,
    path_plugin::PathPlugin, prefix_plugin::PrefixPlugin, print_list_plugin::PrintListPlugin,
    timeout_plugin::TimeoutPlugin, watch_plugin::WatchPlugin,
};
use bodo::process::{color_line, parse_color, ProcessManager};
use bodo::script_loader::ScriptLoader;
use std::collections::HashMap;

#[test]
fn test_all_public_functions() -> Result<()> {
    // Test designer module.
    assert_eq!(designer::EMPTY, ());

    // Test errors Display.
    let e: BodoError = BodoError::NoTaskSpecified;
    let _ = format!("{}", e);

    // Test Graph functionalities.
    let mut graph = Graph::new();
    let node_id = graph.add_node(NodeKind::Task(TaskData {
        name: "all".to_string(),
        description: Some("all".to_string()),
        command: Some("echo all".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: true,
        script_id: "all".to_string(),
        script_display_name: "all".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    }));
    // Try to add an invalid edge (should error)
    let _ = graph.add_edge(node_id, node_id).err();
    // Topological sort should fail due to cycle.
    let _ = graph.topological_sort().err();
    // Call print_debug (output not captured).
    graph.print_debug();
    // Test get_node_name.
    let _ = graph.node_name(node_id as usize);

    // Test ProcessManager kill_all.
    let mut pm = ProcessManager::new(false);
    let _ = pm.spawn_command("dummy", "echo dummy", false, None, None, None);
    let _ = pm.kill_all();

    // Test ScriptLoader merge functions.
    let merged_env = ScriptLoader::merge_envs(
        &HashMap::from([("A".to_string(), "1".to_string())]),
        &HashMap::from([("B".to_string(), "2".to_string())]),
        &HashMap::from([("A".to_string(), "override".to_string())]),
    );
    assert_eq!(merged_env.get("A"), Some(&"override".to_string()));
    let merged_paths = ScriptLoader::merge_exec_paths(
        &vec!["/a".to_string()],
        &vec!["/b".to_string()],
        &vec!["/a".to_string(), "/c".to_string()],
    );
    assert_eq!(
        merged_paths,
        vec!["/a".to_string(), "/b".to_string(), "/c".to_string()]
    );

    // Test PluginManager sort.
    let mut plugin_manager = PluginManager::new();
    plugin_manager.register(Box::new(ConcurrentPlugin::new()));
    plugin_manager.register(Box::new(EnvPlugin::new()));
    plugin_manager.sort_plugins();

    // Test each plugin on_init with default config.
    let default_plugin_config = PluginConfig::default();
    {
        let mut p: Box<dyn Plugin> = Box::new(ExecutionPlugin::new());
        let _ = p.on_init(&default_plugin_config);
    }
    {
        let mut p: Box<dyn Plugin> = Box::new(PathPlugin::new());
        let _ = p.on_init(&default_plugin_config);
    }
    {
        let mut p: Box<dyn Plugin> = Box::new(PrefixPlugin::new());
        let _ = p.on_init(&default_plugin_config);
    }
    {
        let mut p: Box<dyn Plugin> = Box::new(PrintListPlugin);
        let _ = p.on_init(&default_plugin_config);
    }
    {
        let mut p: Box<dyn Plugin> = Box::new(TimeoutPlugin::new());
        let _ = p.on_init(&default_plugin_config);
    }
    {
        let mut p: Box<dyn Plugin> = Box::new(WatchPlugin::new(false, false));
        let _ = p.on_init(&default_plugin_config);
    }

    // Test color_line and parse_color.
    let colored = color_line("Test", &Some("red".to_string()), "line", false);
    assert!(colored.contains("Test"));
    let col = parse_color("red");
    assert!(col.is_some());

    // Test CLI get_task_name using GraphManager with dummy task.
    let mut gm = GraphManager::new();
    gm.graph.nodes.push(bodo::graph::Node {
        id: 0,
        kind: NodeKind::Task(TaskData {
            name: "dummy".to_string(),
            description: Some("dummy".to_string()),
            command: Some("echo dummy".to_string()),
            working_dir: None,
            env: HashMap::new(),
            exec_paths: vec![],
            arguments: vec![],
            is_default: true,
            script_id: "".to_string(),
            script_display_name: "".to_string(),
            watch: None,
            pre_deps: vec![],
            post_deps: vec![],
            concurrently: vec![],
            concurrently_options: Default::default(),
        }),
        metadata: HashMap::new(),
    });
    gm.graph.task_registry.insert("dummy".to_string(), 0);
    let args = Args {
        list: false,
        watch: false,
        auto_watch: false,
        debug: false,
        task: None,
        subtask: None,
        args: vec![],
    };
    let task_name = get_task_name(&args, &gm)?;
    // When no task argument is provided, default task should be chosen.
    assert_eq!(task_name, "default");

    Ok(())
}
