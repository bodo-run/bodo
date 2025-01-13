use std::fs;
use tempfile::tempdir;

#[test]
fn test_plugin_integration() {
    let temp_dir = tempdir().unwrap();
    let project_root = temp_dir.path();
    println!("Project root: {}", project_root.display());

    // Create plugin directory and copy bridge script
    let plugin_dir = project_root.join("plugins");
    fs::create_dir_all(&plugin_dir).unwrap();

    let bridge_dir = project_root.join("src/plugin/bridges");
    fs::create_dir_all(&bridge_dir).unwrap();

    // Copy the bridge script from source
    fs::copy(
        "src/plugin/bridges/bodo-plugin-bridge.js",
        bridge_dir.join("bodo-plugin-bridge.js"),
    )
    .unwrap();

    // Make bridge script executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&bridge_dir.join("bodo-plugin-bridge.js"))
            .unwrap()
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&bridge_dir.join("bodo-plugin-bridge.js"), perms).unwrap();
    }

    // Create a simple JavaScript plugin
    let plugin_path = plugin_dir.join("my-logger.js");
    println!("Plugin path: {}", plugin_path.display());

    fs::write(
        &plugin_path,
        r#"
module.exports = {
    onBeforeTaskRun: (opts) => {
        console.log(`[Logger] Starting task: ${opts.taskName}`);
    },
    onAfterTaskRun: (opts) => {
        console.log(`[Logger] Finished task: ${opts.taskName} with status: ${opts.status}`);
    },
    onCommandReady: (opts) => {
        console.log(`[Logger] Running command: ${opts.command}`);
    }
};
"#,
    )
    .unwrap();

    // Create a global bodo.yaml file
    fs::write(
        project_root.join("bodo.yaml"),
        format!(
            r#"
plugins:
  - {}
"#,
            plugin_path.display()
        ),
    )
    .unwrap();

    // Create a script that uses the plugin
    let script_dir = project_root.join("scripts").join("plugin-test");
    fs::create_dir_all(&script_dir).unwrap();

    let script_content = r#"
name: Plugin Test Script
description: Test plugin functionality
default_task:
  command: echo "Testing plugin"
  description: Default plugin test task
"#;

    let script_path = script_dir.join("script.yaml");
    fs::write(&script_path, script_content).unwrap();

    println!(
        "Current dir: {}",
        std::env::current_dir().unwrap().display()
    );

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_bodo"))
        .arg("plugin-test")
        .current_dir(&project_root)
        .output()
        .unwrap_or_else(|e| panic!("Failed to execute command: {}", e));

    assert!(
        output.status.success(),
        "Unexpected failure.\ncode={:?}\nstdout=```{}```\nstderr=```{}```\ncommand=`cd {:?} && {:?} \"plugin-test\"`",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
        project_root,
        env!("CARGO_BIN_EXE_bodo"),
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[Logger] Starting task: plugin-test"));
    assert!(stdout.contains("Testing plugin"));
}

#[test]
fn test_bash_plugin_integration() {
    let temp_dir = tempdir().unwrap();
    let project_root = temp_dir.path();
    println!("Project root: {}", project_root.display());

    // Create plugin directory and copy bridge script
    let plugin_dir = project_root.join("plugins");
    fs::create_dir_all(&plugin_dir).unwrap();

    let bridge_dir = project_root.join("src/plugin/bridges");
    fs::create_dir_all(&bridge_dir).unwrap();

    // Copy the bridge script from source
    fs::copy(
        "src/plugin/bridges/bodo-plugin-bridge.sh",
        bridge_dir.join("bodo-plugin-bridge.sh"),
    )
    .unwrap();

    // Make bridge script executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&bridge_dir.join("bodo-plugin-bridge.sh"))
            .unwrap()
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&bridge_dir.join("bodo-plugin-bridge.sh"), perms).unwrap();
    }

    // Create a simple Bash plugin
    let plugin_path = plugin_dir.join("test-logger.sh");
    println!("Plugin path: {}", plugin_path.display());

    fs::write(
        &plugin_path,
        r#"#!/bin/bash

on_before_task_run() {
    local task_name="$1"
    local cwd="$2"
    echo "[Bash Logger] Starting task: $task_name in $cwd"
}

on_after_task_run() {
    local task_name="$1"
    local status="$2"
    echo "[Bash Logger] Finished task: $task_name with status: $status"
}

on_command_ready() {
    local command="$1"
    local task_name="$2"
    echo "[Bash Logger] Running command: $command for task: $task_name"
}
"#,
    )
    .unwrap();

    // Make plugin executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&plugin_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&plugin_path, perms).unwrap();
    }

    // Create a global bodo.yaml file
    fs::write(
        project_root.join("bodo.yaml"),
        format!(
            r#"
plugins:
  - {}
"#,
            plugin_path.display()
        ),
    )
    .unwrap();

    // Create a script that uses the plugin
    let script_dir = project_root.join("scripts").join("bash-plugin-test");
    fs::create_dir_all(&script_dir).unwrap();

    let script_content = r#"
name: Bash Plugin Test Script
description: Test bash plugin functionality
default_task:
  command: echo "Testing bash plugin"
  description: Default bash plugin test task
"#;

    let script_path = script_dir.join("script.yaml");
    fs::write(&script_path, script_content).unwrap();

    println!(
        "Current dir: {}",
        std::env::current_dir().unwrap().display()
    );

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_bodo"))
        .arg("bash-plugin-test")
        .current_dir(&project_root)
        .output()
        .unwrap_or_else(|e| panic!("Failed to execute command: {}", e));

    assert!(
        output.status.success(),
        "Unexpected failure.\ncode={:?}\nstdout=```{}```\nstderr=```{}```\ncommand=`cd {:?} && {:?} \"bash-plugin-test\"`",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
        project_root,
        env!("CARGO_BIN_EXE_bodo"),
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[Bash Logger] Starting task: bash-plugin-test"));
    assert!(stdout.contains("Testing bash plugin"));
}
