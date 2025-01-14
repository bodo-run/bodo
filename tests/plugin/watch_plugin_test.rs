use bodo::config::{ConcurrentItem, ConcurrentlyOptions, TaskConfig};
use bodo::env::EnvManager;
use bodo::plugin::PluginManager;
use bodo::plugins::watch_plugin::WatchPlugin;
use bodo::prompt::PromptManager;
use bodo::task::TaskManager;

use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::Write;
use tempfile::tempdir;
use tokio;

#[tokio::test]
async fn test_watch_mode_triggers_task_rerun() -> Result<(), Box<dyn Error>> {
    // Create a temp directory
    let temp = tempdir()?;
    let watched_file_path = temp.path().join("watched_file.txt");

    // Write initial content
    {
        let mut file = File::create(&watched_file_path)?;
        writeln!(file, "Initial content")?;
    }

    // Create task config
    let my_task = ConcurrentItem::Command {
        command: format!(
            "echo 'Re-running due to file change: {}'",
            watched_file_path.display()
        ),
        name: Some("my_task".to_string()),
        output: None,
    };

    let config = TaskConfig {
        concurrently: Some(vec![my_task]),
        concurrently_options: Some(ConcurrentlyOptions {
            fail_fast: false,
            timeout: Some(10),
        }),
        description: Some("Watch-based task".into()),
        ..Default::default()
    };

    // Setup plugin manager with watch plugin
    let mut plugin_manager = PluginManager::new();
    let mut watch_plugin = WatchPlugin::new();
    watch_plugin.watch_file(&watched_file_path);
    plugin_manager.register(Box::new(watch_plugin));

    let env_manager = EnvManager::new();
    let prompt_manager = PromptManager::new();
    let mut task_manager = TaskManager::new(config, env_manager, plugin_manager, prompt_manager);

    // Initial run
    let result = task_manager.run_task("watch_task_init");
    assert!(result.is_ok(), "Initial run should succeed");

    // Modify watched file
    {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&watched_file_path)?;
        writeln!(file, "Additional content to trigger watch")?;
    }

    // Allow time for watch event
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Check run count
    let run_counter = task_manager.plugin_manager().get_watch_run_count();
    assert_eq!(
        run_counter, 2,
        "Expected two runs: initial + one triggered by file change"
    );

    Ok(())
}

#[tokio::test]
async fn test_watch_mode_concurrency_respected() -> Result<(), Box<dyn Error>> {
    let temp = tempdir()?;
    let file_a = temp.path().join("fileA.txt");
    let file_b = temp.path().join("fileB.txt");
    std::fs::write(&file_a, "A")?;
    std::fs::write(&file_b, "B")?;

    let task_a = ConcurrentItem::Command {
        command: format!("echo 'Handling file A: {}'", file_a.display()),
        name: Some("taskA".into()),
        output: None,
    };
    let task_b = ConcurrentItem::Command {
        command: format!("echo 'Handling file B: {}'", file_b.display()),
        name: Some("taskB".into()),
        output: None,
    };

    let config = TaskConfig {
        concurrently: Some(vec![task_a, task_b]),
        concurrently_options: Some(ConcurrentlyOptions {
            fail_fast: false,
            timeout: None,
        }),
        ..Default::default()
    };

    let mut plugin_manager = PluginManager::new();
    let mut watch_plugin = WatchPlugin::new();
    watch_plugin.watch_file(&file_a);
    watch_plugin.watch_file(&file_b);
    plugin_manager.register(Box::new(watch_plugin));

    let env_manager = EnvManager::new();
    let prompt_manager = PromptManager::new();
    let mut task_manager = TaskManager::new(config, env_manager, plugin_manager, prompt_manager);

    let result = task_manager.run_task("initial_watch_run");
    assert!(result.is_ok(), "Initial concurrent run should succeed");

    {
        let mut file = OpenOptions::new().write(true).append(true).open(&file_a)?;
        writeln!(file, "\nTrigger concurrency again!")?;
    }

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let run_counter = task_manager.plugin_manager().get_watch_run_count();
    assert_eq!(run_counter, 2, "Expected second run due to fileA change");

    Ok(())
}
