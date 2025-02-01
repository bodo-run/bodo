use bodo::{
    errors::BodoError,
    graph::{Graph, NodeKind, TaskData},
    plugin::{PluginConfig, PluginManager},
    plugins::{execution_plugin::ExecutionPlugin, watch_plugin::WatchPlugin},
    Result,
};
use serde_json::json;
use std::{collections::HashMap, fs, time::Duration};
use tempfile::tempdir;
use tokio::time::sleep;

#[tokio::test]
async fn test_watch_basic() -> Result<()> {
    let dir = tempdir()?;
    let test_file = dir.path().join("test.txt");
    fs::write(&test_file, "initial content")?;

    let mut graph = Graph::new();
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "watch_test".into(),
        description: Some("Test watch functionality".into()),
        command: Some("echo File changed".into()),
        working_dir: None,
        env: HashMap::new(),
        is_default: false,
        script_id: "test_script".to_string(),
        script_display_name: "Test".to_string(),
    }));

    let node = &mut graph.nodes[task_id as usize];
    node.metadata.insert(
        "watch".to_string(),
        json!({
            "patterns": [test_file.to_string_lossy().to_string()],
            "debounce_ms": 100,
            "ignore_patterns": []
        })
        .to_string(),
    );

    let mut manager = PluginManager::new();
    manager.register(Box::new(WatchPlugin::new()));
    manager.register(Box::new(ExecutionPlugin));

    manager
        .run_lifecycle(&mut graph, Some(PluginConfig::default()))
        .await?;

    sleep(Duration::from_millis(200)).await;
    fs::write(&test_file, "modified content")?;
    sleep(Duration::from_millis(200)).await;

    Ok(())
}

#[tokio::test]
async fn test_watch_ignore_patterns() -> Result<()> {
    let dir = tempdir()?;
    let test_file = dir.path().join("test.txt");
    let ignored_file = dir.path().join("ignored.tmp");
    fs::write(&test_file, "initial content")?;
    fs::write(&ignored_file, "initial content")?;

    let mut graph = Graph::new();
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "watch_test".into(),
        description: Some("Test watch functionality".into()),
        command: Some("echo File changed".into()),
        working_dir: None,
        env: HashMap::new(),
        is_default: false,
        script_id: "test_script".to_string(),
        script_display_name: "Test".to_string(),
    }));

    let node = &mut graph.nodes[task_id as usize];
    node.metadata.insert(
        "watch".to_string(),
        json!({
            "patterns": [dir.path().to_string_lossy().to_string()],
            "debounce_ms": 100,
            "ignore_patterns": [".tmp"]
        })
        .to_string(),
    );

    let mut manager = PluginManager::new();
    manager.register(Box::new(WatchPlugin::new()));
    manager.register(Box::new(ExecutionPlugin));

    manager
        .run_lifecycle(&mut graph, Some(PluginConfig::default()))
        .await?;

    sleep(Duration::from_millis(200)).await;
    fs::write(&test_file, "modified content")?;
    fs::write(&ignored_file, "modified content")?;
    sleep(Duration::from_millis(200)).await;

    Ok(())
}

#[tokio::test]
async fn test_watch_debounce() -> Result<()> {
    let dir = tempdir()?;
    let test_file = dir.path().join("test.txt");
    fs::write(&test_file, "initial content")?;

    let mut graph = Graph::new();
    let task_id = graph.add_node(NodeKind::Task(TaskData {
        name: "watch_test".into(),
        description: Some("Test watch functionality".into()),
        command: Some("echo File changed".into()),
        working_dir: None,
        env: HashMap::new(),
        is_default: false,
        script_id: "test_script".to_string(),
        script_display_name: "Test".to_string(),
    }));

    let node = &mut graph.nodes[task_id as usize];
    node.metadata.insert(
        "watch".to_string(),
        json!({
            "patterns": [test_file.to_string_lossy().to_string()],
            "debounce_ms": 1000,
            "ignore_patterns": []
        })
        .to_string(),
    );

    let mut manager = PluginManager::new();
    manager.register(Box::new(WatchPlugin::new()));
    manager.register(Box::new(ExecutionPlugin));

    manager
        .run_lifecycle(&mut graph, Some(PluginConfig::default()))
        .await?;

    sleep(Duration::from_millis(200)).await;
    for i in 0..5 {
        fs::write(&test_file, format!("content {}", i))?;
        sleep(Duration::from_millis(100)).await;
    }
    sleep(Duration::from_millis(1000)).await;

    Ok(())
}

#[tokio::test]
async fn test_multiple_watchers() -> Result<()> {
    let dir = tempdir()?;
    let file1 = dir.path().join("file1.txt");
    let file2 = dir.path().join("file2.txt");
    fs::write(&file1, "initial content")?;
    fs::write(&file2, "initial content")?;

    let mut graph = Graph::new();

    // Task 1 watching file1
    let task1_id = graph.add_node(NodeKind::Task(TaskData {
        name: "watch_test_1".into(),
        description: Some("Test watch functionality 1".into()),
        command: Some("echo 'File 1 changed'".into()),
        working_dir: None,
        env: HashMap::new(),
        is_default: false,
        script_id: "test_script_1".to_string(),
        script_display_name: "Test 1".to_string(),
    }));

    // Task 2 watching file2
    let task2_id = graph.add_node(NodeKind::Task(TaskData {
        name: "watch_test_2".into(),
        description: Some("Test watch functionality 2".into()),
        command: Some("echo 'File 2 changed'".into()),
        working_dir: None,
        env: HashMap::new(),
        is_default: false,
        script_id: "test_script_2".to_string(),
        script_display_name: "Test 2".to_string(),
    }));

    // Configure watchers
    let node1 = &mut graph.nodes[task1_id as usize];
    node1.metadata.insert(
        "watch".to_string(),
        json!({
            "patterns": [file1.to_string_lossy().to_string()],
            "debounce_ms": 100,
            "ignore_patterns": []
        })
        .to_string(),
    );

    let node2 = &mut graph.nodes[task2_id as usize];
    node2.metadata.insert(
        "watch".to_string(),
        json!({
            "patterns": [file2.to_string_lossy().to_string()],
            "debounce_ms": 100,
            "ignore_patterns": []
        })
        .to_string(),
    );

    let mut manager = PluginManager::new();
    manager.register(Box::new(WatchPlugin::new()));
    manager.register(Box::new(ExecutionPlugin));

    manager
        .run_lifecycle(&mut graph, Some(PluginConfig::default()))
        .await?;

    // Wait for watchers to be set up
    sleep(Duration::from_millis(200)).await;

    // Modify both files
    fs::write(&file1, "modified content 1")?;
    fs::write(&file2, "modified content 2")?;

    // Wait for both watchers to trigger
    sleep(Duration::from_millis(300)).await;

    Ok(())
}

#[tokio::test]
async fn test_watch_plugin() -> Result<()> {
    let temp = tempdir()?;
    let project_root = temp.path();

    // Create test files
    fs::create_dir_all(project_root.join("scripts"))?;
    fs::write(
        project_root.join("scripts/script.yaml"),
        r#"
tasks:
  test:
    command: "echo 'test'"
"#,
    )?;

    let mut manager = PluginManager::new();
    let mut graph = Graph::new();

    // Register plugins
    manager.register(Box::new(ExecutionPlugin));

    // Run lifecycle with default config
    manager
        .run_lifecycle(&mut graph, Some(PluginConfig::default()))
        .await?;

    // Wait a bit for file changes to be detected
    sleep(Duration::from_millis(100)).await;

    Ok(())
}

#[tokio::test]
async fn test_watch_plugin_invalid_config() -> Result<()> {
    let mut graph = Graph::new();

    // Create a task
    let task = TaskData {
        name: "test".to_string(),
        description: Some("Test task".to_string()),
        command: Some("echo test".to_string()),
        working_dir: None,
        is_default: false,
        script_id: "test_script".to_string(),
        script_display_name: "Test".to_string(),
        env: HashMap::new(),
    };
    let task_id = graph.add_node(NodeKind::Task(task));

    // Add invalid watch metadata
    let node = &mut graph.nodes[task_id as usize];
    node.metadata
        .insert("watch".to_string(), "invalid json".to_string());

    // Setup plugins
    let mut manager = PluginManager::new();
    manager.register(Box::new(WatchPlugin::new()));

    // Run plugins to process metadata
    let result = manager
        .run_lifecycle(
            &mut graph,
            Some(PluginConfig {
                fail_fast: false,
                watch: true,
                list: false,
                options: None,
            }),
        )
        .await;

    assert!(result.is_err());
    assert!(matches!(result, Err(BodoError::PluginError(_))));

    Ok(())
}
