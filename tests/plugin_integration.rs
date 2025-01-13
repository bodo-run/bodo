use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use tempfile::tempdir;

/// Tests that a plugin is recognized and executed as described in the README
#[test]
fn test_plugin_integration() {
    let temp = tempdir().unwrap();
    let project_root = temp.path();

    // Create plugin directory and bridge directory
    let plugin_dir = project_root.join("plugins");
    let bridge_dir = project_root.join("src").join("plugin").join("bridges");
    fs::create_dir_all(&plugin_dir).unwrap();
    fs::create_dir_all(&bridge_dir).unwrap();

    // Copy bridge script from source
    let bridge_script = r#"
const fs = require('fs');
const path = require('path');

function loadPlugin(pluginPath) {
    const absPath = path.resolve(pluginPath);
    console.error("[DEBUG] Loading plugin from:", absPath);
    try {
        const plugin = require(absPath);
        console.error("[DEBUG] Plugin loaded successfully");
        console.error("[DEBUG] Plugin exports:", Object.keys(plugin));
        return plugin;
    } catch (err) {
        console.error("[DEBUG] Failed to load plugin:", err);
        console.error("[DEBUG] Current directory:", process.cwd());
        console.error("[DEBUG] Plugin file exists:", fs.existsSync(absPath));
        return null;
    }
}

const pluginFile = process.env.BODO_PLUGIN_FILE;
const opts = process.env.BODO_OPTS ? JSON.parse(process.env.BODO_OPTS) : {};

console.error("[DEBUG] Plugin file:", pluginFile);
console.error("[DEBUG] Options:", JSON.stringify(opts));

if (!pluginFile) {
    console.error("BODO_PLUGIN_FILE environment variable not set");
    process.exit(1);
}

const plugin = loadPlugin(pluginFile);
if (!plugin) {
    console.error("Failed to load plugin");
    process.exit(1);
}

if (plugin[opts.hook]) {
    plugin[opts.hook](opts);
} else {
    console.error("[DEBUG] Available hooks:", Object.keys(plugin));
    console.error(`Plugin does not export a '${opts.hook}' function`);
    process.exit(1);
}
"#;
    fs::write(bridge_dir.join("bodo-plugin-bridge.js"), bridge_script).unwrap();

    // Make the bridge script executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&bridge_dir.join("bodo-plugin-bridge.js"))
            .unwrap()
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&bridge_dir.join("bodo-plugin-bridge.js"), perms).unwrap();
    }

    // Place a sample JS plugin
    let plugin_dir = project_root.join("plugins");
    fs::create_dir_all(&plugin_dir).unwrap();

    // Write plugin content
    let plugin_content = r#"
module.exports = {
    onBeforeTaskRun: (opts) => {
        console.error("[TEST_PLUGIN] onBeforeTaskRun triggered for " + opts.taskName);
    },
    onAfterTaskRun: (opts) => {
        console.error("[TEST_PLUGIN] onAfterTaskRun triggered for " + opts.taskName);
    },
    onResolveCommand: (opts) => {
        console.error("[TEST_PLUGIN] onResolveCommand triggered for task");
        return opts.task;
    },
    onCommandReady: (opts) => {
        console.error("[TEST_PLUGIN] onCommandReady triggered for " + opts.taskName);
        return opts.command;
    },
    onBodoExit: (opts) => {
        console.error("[TEST_PLUGIN] onBodoExit triggered with code " + opts.exitCode);
    }
};
"#;
    let plugin_path = plugin_dir.join("my-logger.js");
    fs::write(&plugin_path, plugin_content).unwrap();

    // Write bodo.yaml to register plugin
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

    // Create test directory with script
    let test_dir = project_root.join("scripts").join("plugin-test");
    fs::create_dir_all(&test_dir).unwrap();
    fs::write(
        test_dir.join("script.yaml"),
        r#"
name: Plugin Test
description: Testing plugin hooks
defaultTask:
  command: echo "Running a plugin test"
"#,
    )
    .unwrap();

    // Print debug information
    eprintln!("Project root: {}", project_root.display());
    eprintln!(
        "Bridge script path: {}",
        bridge_dir.join("bodo-plugin-bridge.js").display()
    );
    eprintln!("Plugin path: {}", plugin_path.display());
    eprintln!(
        "Current dir: {}",
        std::env::current_dir().unwrap().display()
    );

    // Run the command and check output
    Command::cargo_bin("bodo")
        .unwrap()
        .current_dir(&project_root)
        .arg("plugin-test")
        .assert()
        .success()
        .stderr(contains(
            "[TEST_PLUGIN] onBeforeTaskRun triggered for plugin-test",
        ))
        .stderr(contains(
            "[TEST_PLUGIN] onAfterTaskRun triggered for plugin-test",
        ))
        .stdout(contains("Running a plugin test"));
}
