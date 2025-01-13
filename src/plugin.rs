use std::process::Command;
use crate::config::BodoConfig;

pub trait BodoPlugin {
    fn on_before_run(&self, task_name: &str);
    fn on_after_run(&self, task_name: &str);
}

pub struct PluginManager<'a> {
    config: &'a BodoConfig,
}

impl<'a> PluginManager<'a> {
    pub fn new(config: &'a BodoConfig) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    fn create_test_plugin(content: &str, extension: &str) -> PathBuf {
        let mut temp_path = std::env::temp_dir();
        temp_path.push(format!("test_plugin.{}", extension));
        
        let mut file = File::create(&temp_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        
        #[cfg(unix)]
        std::fs::set_permissions(&temp_path, std::fs::Permissions::from_mode(0o755)).unwrap();
        
        temp_path
    }

    fn cleanup_temp_file(path: PathBuf) {
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_plugin_manager_creation() {
        let config = BodoConfig::default();
        let plugin_manager = PluginManager::new(&config);
        assert!(plugin_manager.config.plugins.is_none());
    }

    #[test]
    fn test_run_shell_plugin() {
        let plugin_content = r#"#!/bin/sh
echo "Running plugin for task: $1"
exit 0
"#;
        let plugin_path = create_test_plugin(plugin_content, "sh");
        
        let config = BodoConfig {
            tasks: None,
            env_files: None,
            executable_map: None,
            max_concurrency: None,
            plugins: Some(vec![plugin_path.to_string_lossy().to_string()]),
        };
        
        let plugin_manager = PluginManager::new(&config);
        plugin_manager.run_plugins_for_task("test_task");
        
        cleanup_temp_file(plugin_path);
    }

    #[test]
    fn test_run_js_plugin() {
        let plugin_content = r#"
console.log('Running plugin for task:', process.argv[2]);
process.exit(0);
"#;
        let plugin_path = create_test_plugin(plugin_content, "js");
        
        let config = BodoConfig {
            tasks: None,
            env_files: None,
            executable_map: None,
            max_concurrency: None,
            plugins: Some(vec![plugin_path.to_string_lossy().to_string()]),
        };
        
        let plugin_manager = PluginManager::new(&config);
        plugin_manager.run_plugins_for_task("test_task");
        
        cleanup_temp_file(plugin_path);
    }

    #[test]
    fn test_no_plugins() {
        let config = BodoConfig {
            tasks: None,
            env_files: None,
            executable_map: None,
            max_concurrency: None,
            plugins: None,
        };
        
        let plugin_manager = PluginManager::new(&config);
        // Should not panic when no plugins are configured
        plugin_manager.run_plugins_for_task("test_task");
    }
} 