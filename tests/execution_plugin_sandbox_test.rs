use bodo::graph::{Graph, Node, NodeKind, TaskData};
use bodo::plugin::{Plugin, PluginConfig, SideEffect};
use bodo::plugins::execution_plugin::ExecutionPlugin;
use std::collections::HashMap;

#[test]
fn test_execution_plugin_sandbox_integration() {
    let mut plugin = ExecutionPlugin::new();

    // Configure plugin for dry-run mode
    let config = PluginConfig {
        fail_fast: false,
        watch: false,
        list: false,
        dry_run: true,
        options: Some({
            let mut options = serde_json::Map::new();
            options.insert(
                "task".to_string(),
                serde_json::Value::String("test_task".to_string()),
            );
            options
        }),
    };

    plugin
        .on_init(&config)
        .expect("Plugin initialization should succeed");
    assert!(plugin.dry_run, "Plugin should be in dry-run mode");
}

#[test]
fn test_enhanced_side_effect_analysis() {
    let plugin = ExecutionPlugin::new();
    let working_dir = std::path::Path::new("/tmp");
    let env = HashMap::new();

    // Test various commands that should be detected
    let test_cases = vec![
        ("echo 'test' > file.txt", "file write"),
        ("touch newfile.txt", "file creation"),
        ("cat existing.txt", "file read"),
        ("rm oldfile.txt", "file deletion"),
        ("mkdir newdir", "directory creation"),
        ("curl http://example.com", "network request"),
        ("sed -i 's/old/new/g' file.txt", "file modification"),
    ];

    for (command, description) in test_cases {
        let side_effects = plugin.analyze_side_effects(command, working_dir, &env);

        // Should at least detect process spawn
        let has_process_spawn = side_effects
            .iter()
            .any(|effect| matches!(effect, SideEffect::ProcessSpawn(_)));

        assert!(
            has_process_spawn,
            "Should detect process spawn for {}: {}",
            description, command
        );
        assert!(
            !side_effects.is_empty(),
            "Should detect side effects for {}: {}",
            description,
            command
        );
    }
}

#[test]
fn test_fallback_pattern_analysis() {
    let plugin = ExecutionPlugin::new();
    let working_dir = std::path::Path::new("/tmp");

    // Test the fallback pattern-based analysis directly
    let side_effects =
        plugin.analyze_side_effects_fallback("echo 'test' > output.txt", working_dir);

    // Should detect process spawn and file write
    let has_process_spawn = side_effects
        .iter()
        .any(|effect| matches!(effect, SideEffect::ProcessSpawn(_)));
    let has_file_write = side_effects
        .iter()
        .any(|effect| matches!(effect, SideEffect::FileWrite(_)));

    assert!(has_process_spawn, "Should detect process spawn");
    assert!(has_file_write, "Should detect file write");
}

#[test]
fn test_file_path_extraction() {
    let plugin = ExecutionPlugin::new();

    // Test various file path extraction scenarios
    let test_cases = vec![
        ("echo 'test' > file.txt", Some("file.txt".to_string())),
        ("cat input.txt", Some("input.txt".to_string())),
        ("touch newfile.txt", Some("newfile.txt".to_string())),
        ("echo 'test'", None), // No file operation
    ];

    for (command, expected) in test_cases {
        let result = plugin.extract_file_path_from_command(command);
        assert_eq!(
            result, expected,
            "File path extraction failed for: {}",
            command
        );
    }
}

#[test]
fn test_rm_command_path_extraction() {
    let plugin = ExecutionPlugin::new();

    let test_cases = vec![
        ("rm file.txt", Some("file.txt".to_string())),
        ("rm -rf directory/", Some("directory/".to_string())),
        ("rm -f file1.txt file2.txt", Some("file2.txt".to_string())), // Gets last file
        ("rm", None),                                                 // No file specified
    ];

    for (command, expected) in test_cases {
        let result = plugin.extract_file_path_from_rm_command(command);
        assert_eq!(
            result, expected,
            "RM path extraction failed for: {}",
            command
        );
    }
}

#[test]
fn test_mkdir_command_path_extraction() {
    let plugin = ExecutionPlugin::new();

    let test_cases = vec![
        ("mkdir newdir", Some("newdir".to_string())),
        ("mkdir -p path/to/dir", Some("path/to/dir".to_string())),
        ("mkdir", None), // No directory specified
    ];

    for (command, expected) in test_cases {
        let result = plugin.extract_file_path_from_mkdir_command(command);
        assert_eq!(
            result, expected,
            "MKDIR path extraction failed for: {}",
            command
        );
    }
}

#[test]
fn test_sed_command_path_extraction() {
    let plugin = ExecutionPlugin::new();

    let test_cases = vec![
        (
            "sed -i 's/old/new/g' file.txt",
            Some("file.txt".to_string()),
        ),
        ("sed 's/old/new/g' input.txt", Some("input.txt".to_string())),
        ("sed -i", None), // No file specified
    ];

    for (command, expected) in test_cases {
        let result = plugin.extract_file_path_from_sed_command(command);
        assert_eq!(
            result, expected,
            "SED path extraction failed for: {}",
            command
        );
    }
}

#[test]
fn test_duration_estimation() {
    let plugin = ExecutionPlugin::new();

    let test_cases = vec![
        ("echo 'hello'", 1), // Simple command
        ("sleep 10", 5),     // Sleep command
        ("npm install", 30), // Build command
        ("cargo build", 30), // Build command
        ("cargo test", 10),  // Test command
    ];

    for (command, expected_secs) in test_cases {
        let duration = plugin.estimate_duration(command);
        assert_eq!(
            duration.as_secs(),
            expected_secs,
            "Duration estimation failed for: {}",
            command
        );
    }
}

#[test]
fn test_dry_run_with_sandbox_integration() {
    let mut plugin = ExecutionPlugin::new();
    plugin.dry_run = true;
    plugin.task_name = Some("test_task".to_string());

    // Create a simple graph with a task
    let mut graph = Graph::new();
    let task_data = TaskData {
        name: "test_task".to_string(),
        description: None,
        command: Some("echo 'test' > output.txt".to_string()),
        working_dir: Some("/tmp".to_string()),
        env: HashMap::new(),
        exec_paths: vec![],
        is_default: false,
        script_id: "test".to_string(),
        script_display_name: "test".to_string(),
        watch: None,
        arguments: vec![],
        pre_deps: vec![],
        post_deps: vec![],
        concurrently: vec![],
        concurrently_options: Default::default(),
    };

    let node = Node {
        id: 0,
        kind: NodeKind::Task(task_data),
        metadata: HashMap::new(),
    };

    graph.nodes.push(node);
    graph.task_registry.insert("test_task".to_string(), 0);

    // Execute dry run
    let result = plugin.on_after_run(&mut graph);

    // Should succeed (output will be printed to stdout)
    assert!(result.is_ok(), "Dry run should succeed");
}

#[test]
fn test_enhanced_dry_run_output() {
    let plugin = ExecutionPlugin::new();

    // Create mock dry run reports
    let reports = vec![bodo::plugin::DryRunReport {
        command: "echo 'test' > file.txt".to_string(),
        environment: HashMap::new(),
        working_directory: std::path::PathBuf::from("/tmp"),
        dependencies: vec![],
        estimated_duration: Some(std::time::Duration::from_secs(1)),
        side_effects: vec![
            SideEffect::ProcessSpawn("echo 'test' > file.txt".to_string()),
            SideEffect::FileWrite(std::path::PathBuf::from("/tmp/file.txt")),
        ],
    }];

    // Test display (this will print to stdout)
    let result = plugin.display_dry_run_results(&reports);
    assert!(result.is_ok(), "Display should succeed");
}

#[test]
fn test_environment_variable_expansion() {
    let plugin = ExecutionPlugin::new();
    let mut env = HashMap::new();
    env.insert("HOME".to_string(), "/home/user".to_string());
    env.insert("PROJECT".to_string(), "myproject".to_string());

    let test_cases = vec![
        ("echo $HOME", "echo /home/user"),
        ("cd ${PROJECT}/src", "cd myproject/src"),
        ("echo $$", "echo $"),                      // Escaped dollar
        ("echo $UNDEFINED", "echo $UNDEFINED"),     // Undefined variable
        ("echo ${UNDEFINED}", "echo ${UNDEFINED}"), // Undefined variable with braces
    ];

    for (input, expected) in test_cases {
        let result = plugin.expand_env_vars(input, &env);
        assert_eq!(
            result, expected,
            "Environment expansion failed for: {}",
            input
        );
    }
}

#[test]
fn test_complex_command_analysis() {
    let plugin = ExecutionPlugin::new();
    let working_dir = std::path::Path::new("/tmp");
    let env = HashMap::new();

    // Test complex command with multiple operations
    let command = "mkdir -p output && echo 'data' > output/file.txt && cat output/file.txt";
    let side_effects = plugin.analyze_side_effects(command, working_dir, &env);

    // Should detect process spawn at minimum
    let has_process_spawn = side_effects
        .iter()
        .any(|effect| matches!(effect, SideEffect::ProcessSpawn(_)));

    assert!(
        has_process_spawn,
        "Should detect process spawn for complex command"
    );
    assert!(
        !side_effects.is_empty(),
        "Should detect side effects for complex command"
    );
}
