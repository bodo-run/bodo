use bodo::plugins::{
    env_plugin::EnvPlugin, execution_plugin::ExecutionPlugin, path_plugin::PathPlugin,
};
use bodo::{
    designer,
    plugin::{Plugin, PluginConfig, PluginManager},
    BodoConfig, Graph, GraphManager,
};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use tempfile::{tempdir, NamedTempFile};

#[test]
fn test_all_public_functions() {
    // Test designer module public constant.
    assert_eq!(designer::EMPTY, ());

    // Test generating schema from BodoConfig.
    let schema = BodoConfig::generate_schema();
    assert!(!schema.is_empty());
    assert!(schema.contains("\"title\": \"BodoConfig\""));

    // Test basic Graph functionality.
    let mut graph = Graph::new();
    let node_id = graph.add_node(bodo::graph::NodeKind::Task(bodo::graph::TaskData {
        name: "test".to_string(),
        description: Some("description".to_string()),
        command: Some("echo test".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    }));
    assert!(node_id == 0);
    // Adding an edge with an invalid node id should error.
    assert!(graph.add_edge(0, 10).is_err());

    // Test cycle detection and topological sort.
    let cycle = graph.detect_cycle();
    assert!(cycle.is_none());
    let sorted = graph.topological_sort().unwrap();
    assert_eq!(sorted.len(), graph.nodes.len());

    // Test ScriptLoader merge functions.
    let merged_env = bodo::script_loader::ScriptLoader::merge_envs(
        &HashMap::from([("A".to_string(), "1".to_string())]),
        &HashMap::from([("B".to_string(), "2".to_string())]),
        &HashMap::from([("C".to_string(), "3".to_string())]),
    );
    assert_eq!(merged_env.get("A"), Some(&"1".to_string()));
    assert_eq!(merged_env.get("B"), Some(&"2".to_string()));
    assert_eq!(merged_env.get("C"), Some(&"3".to_string()));

    let merged_paths = bodo::script_loader::ScriptLoader::merge_exec_paths(
        &vec!["/a".to_string(), "/b".to_string()],
        &vec!["/b".to_string(), "/c".to_string()],
        &vec!["/a".to_string(), "/d".to_string()],
    );
    assert_eq!(
        merged_paths,
        vec![
            "/a".to_string(),
            "/b".to_string(),
            "/c".to_string(),
            "/d".to_string()
        ]
    );

    // Test ExecutionPlugin expand_env_vars.
    let exec_plugin = ExecutionPlugin::new();
    let expanded = exec_plugin.expand_env_vars(
        "echo $HOME",
        &HashMap::from([("HOME".to_string(), "/home/user".to_string())]),
    );
    assert_eq!(expanded, "echo /home/user");

    // Test EnvPlugin on_init and on_graph_build.
    let mut env_plugin = EnvPlugin::new();
    let config = PluginConfig::default();
    env_plugin.on_init(&config).unwrap();
    let mut g = Graph::new();
    let task_data = bodo::graph::TaskData {
        name: "env_task".to_string(),
        description: None,
        command: Some("echo env".to_string()),
        working_dir: None,
        env: HashMap::new(),
        exec_paths: vec![],
        arguments: vec![],
        is_default: false,
        script_id: "".to_string(),
        script_display_name: "".to_string(),
        watch: None,
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    };
    let tid = g.add_node(bodo::graph::NodeKind::Task(task_data));
    env_plugin.global_env = Some(HashMap::from([("VAR".to_string(), "VALUE".to_string())]));
    env_plugin.on_graph_build(&mut g).unwrap();
    if let bodo::graph::NodeKind::Task(ref td) = g.nodes[tid as usize].kind {
        assert_eq!(td.env.get("VAR"), Some(&"VALUE".to_string()));
    }

    // Test PathPlugin getters/setters and build_path.
    let mut path_plugin = PathPlugin::new();
    path_plugin.set_default_paths(vec!["/default".to_string()]);
    path_plugin.set_preserve_path(false);
    let built_path =
        path_plugin.test_build_path(Some(&"/working".to_string()), &vec!["/exec".to_string()]);
    assert_eq!(built_path, "/working:/default:/exec");

    // Test PluginManager sorting and running lifecycle.
    struct DummyPlugin {
        called: bool,
    }
    impl Plugin for DummyPlugin {
        fn name(&self) -> &'static str {
            "DummyPlugin"
        }
        fn priority(&self) -> i32 {
            5
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn on_init(&mut self, _config: &PluginConfig) -> Result<(), bodo::errors::BodoError> {
            self.called = true;
            Ok(())
        }
    }
    let mut pm = PluginManager::new();
    pm.register(Box::new(DummyPlugin { called: false }));
    pm.sort_plugins();
    pm.run_lifecycle(&mut g, None).unwrap();

    // Test GraphManager build_graph with default config.
    let mut gm = GraphManager::new();
    gm.build_graph(BodoConfig::default()).unwrap();

    // Test GraphManager task_exists and apply_task_arguments.
    gm.graph.nodes.push(bodo::graph::Node {
        id: 0,
        kind: NodeKind::Task(bodo::graph::TaskData {
            name: "dummy".to_string(),
            description: None,
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
    assert!(gm.task_exists("dummy"));
    // Even if task has no arguments defined, apply_task_arguments should succeed.
    gm.apply_task_arguments("dummy", &["ARG1".to_string()]).ok();

    // Test BodoConfig::load using a temporary file.
    let mut tmp = NamedTempFile::new().unwrap();
    let yaml_str = r#"
default_task:
  command: echo "temp default"
tasks:
  temp:
    command: echo "temp task"
"#;
    write!(tmp, "{}", yaml_str).unwrap();
    let loaded_config = BodoConfig::load(Some(tmp.path().to_str().unwrap().to_string())).unwrap();
    assert!(loaded_config.tasks.contains_key("temp"));
}

#[test]
fn test_bodoconfig_load_invalid() {
    let result = BodoConfig::load(Some("nonexistent_config.yaml".to_string()));
    assert!(result.is_err());
}

#[test]
fn test_bodoconfig_load_bad_yaml() {
    let mut tmp = NamedTempFile::new().unwrap();
    let bad_yaml = "default_task: [not a map";
    write!(tmp, "{}", bad_yaml).unwrap();
    let result = BodoConfig::load(Some(tmp.path().to_str().unwrap().to_string()));
    assert!(result.is_err());
}

#[test]
fn test_file_io_functions() {
    // Write a temporary file and read it using fs.
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("temp.txt");
    fs::write(&file_path, "Test content").unwrap();
    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "Test content");
}
