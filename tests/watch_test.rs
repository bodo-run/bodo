use bodo::{
    graph::{Graph, NodeKind, TaskData},
    plugin::{PluginConfig, PluginManager},
    plugins::{execution_plugin::ExecutionPlugin, watch_plugin::WatchPlugin},
    Result,
};
use serde_json::json;
use std::{collections::HashMap, fs, path::PathBuf, time::Duration};
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
        script_name: None,
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
        .run_lifecycle(&mut graph, &PluginConfig::default())
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
        script_name: None,
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
        .run_lifecycle(&mut graph, &PluginConfig::default())
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
        script_name: None,
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
        .run_lifecycle(&mut graph, &PluginConfig::default())
        .await?;

    sleep(Duration::from_millis(200)).await;
    for i in 0..5 {
        fs::write(&test_file, format!("content {}", i))?;
        sleep(Duration::from_millis(100)).await;
    }
    sleep(Duration::from_millis(1000)).await;

    Ok(())
}
