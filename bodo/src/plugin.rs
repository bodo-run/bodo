use std::process::Command;
use crate::config::BodoConfig;

pub trait BodoPlugin {
    fn on_before_run(&self, task_name: &str);
    fn on_after_run(&self, task_name: &str);
}

pub struct PluginManager {
    config: BodoConfig,
}

impl PluginManager {
    pub fn new(config: BodoConfig) -> Self {
        Self { config }
    }

    pub fn run_plugins_for_task(&self, task_name: &str) {
        if let Some(plugins) = &self.config.plugins {
            for plugin_path in plugins {
                self.run_plugin(plugin_path, task_name);
            }
        }
    }

    fn run_plugin(&self, plugin_path: &str, task_name: &str) {
        println!("[BODO] Running plugin: {} for task: {}", plugin_path, task_name);
        let mut cmd_vec = vec![];

        if plugin_path.ends_with(".ts") {
            cmd_vec.push("npx");
            cmd_vec.push("tsx");
            cmd_vec.push(plugin_path);
        } else if plugin_path.ends_with(".js") {
            cmd_vec.push("node");
            cmd_vec.push(plugin_path);
        } else if plugin_path.ends_with(".sh") {
            cmd_vec.push("sh");
            cmd_vec.push(plugin_path);
        } else {
            // fallback
            cmd_vec.push("sh");
            cmd_vec.push(plugin_path);
        }

        cmd_vec.push(task_name);

        let status = Command::new(cmd_vec[0])
            .args(&cmd_vec[1..])
            .status()
            .expect("[BODO] Failed to spawn plugin");

        if !status.success() {
            eprintln!("[BODO] Plugin process failed with code: {:?}", status.code());
        }
    }
} 