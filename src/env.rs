use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Clone)]
pub struct EnvManager {
    env_vars: HashMap<String, String>,
}

impl EnvManager {
    pub fn new() -> Self {
        EnvManager {
            env_vars: HashMap::new(),
        }
    }

    pub fn merge_env_files(&mut self, env_files: &[String]) {
        for file_path in env_files {
            if let Err(e) = self.load_env_file(file_path) {
                eprintln!("[BODO] Error loading env file {}: {}", file_path, e);
            }
        }
    }

    pub fn inject_exec_paths(&mut self, exec_paths: &[String]) {
        if let Ok(current_path) = std::env::var("PATH") {
            let new_paths: Vec<String> = exec_paths
                .iter()
                .map(|p| {
                    if p.starts_with("./") {
                        std::env::current_dir()
                            .map(|d| d.join(&p[2..]))
                            .and_then(|p| p.canonicalize())
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|_| p.clone())
                    } else {
                        p.clone()
                    }
                })
                .collect();

            let path_separator = if cfg!(windows) { ";" } else { ":" };
            let new_path = format!(
                "{}{}{}",
                new_paths.join(path_separator),
                path_separator,
                current_path
            );

            std::env::set_var("PATH", new_path);
        }
    }

    fn load_env_file(&mut self, file_path: &str) -> std::io::Result<()> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_string();
                let mut value = value.trim().to_string();

                // Remove quotes if present
                if (value.starts_with('"') && value.ends_with('"'))
                    || (value.starts_with('\'') && value.ends_with('\''))
                {
                    value = value[1..value.len() - 1].to_string();
                }

                self.env_vars.insert(key, value);
            }
        }

        Ok(())
    }

    pub fn get_env(&self) -> &HashMap<String, String> {
        &self.env_vars
    }
}

impl Default for EnvManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    fn create_temp_env_file(content: &str) -> PathBuf {
        let mut temp_path = std::env::temp_dir();
        let unique_name = format!(
            "test_{}.env",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        temp_path.push(unique_name);

        let mut file = File::create(&temp_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();

        temp_path
    }

    fn cleanup_temp_file(path: PathBuf) {
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_new_env_manager() {
        let env_manager = EnvManager::new();
        assert!(env_manager.env_vars.is_empty());
    }

    #[test]
    fn test_load_env_file() {
        let content = "TEST_KEY=test_value\n# comment\nTEST_KEY2='quoted value'\n";
        let temp_path = create_temp_env_file(content);

        let mut env_manager = EnvManager::new();
        env_manager.merge_env_files(&[temp_path.to_string_lossy().to_string()]);

        let env_vars = env_manager.get_env();
        assert!(env_vars.contains_key("TEST_KEY"));
        assert!(env_vars.contains_key("TEST_KEY2"));
        assert_eq!(env_vars.get("TEST_KEY").unwrap(), "test_value");
        assert_eq!(env_vars.get("TEST_KEY2").unwrap(), "quoted value");

        cleanup_temp_file(temp_path);
    }

    #[test]
    fn test_load_env_file_with_empty_lines() {
        let content = "\nTEST_KEY=test_value\n\n";
        let temp_path = create_temp_env_file(content);

        let mut env_manager = EnvManager::new();
        env_manager
            .load_env_file(&temp_path.to_string_lossy().to_string())
            .unwrap();

        let env_vars = env_manager.get_env();
        assert_eq!(env_vars.get("TEST_KEY").unwrap(), "test_value");

        cleanup_temp_file(temp_path);
    }

    #[test]
    fn test_inject_exec_paths() {
        let mut env_manager = EnvManager::new();
        let original_path = std::env::var("PATH").unwrap();

        env_manager.inject_exec_paths(&["./bin".to_string()]);

        let new_path = std::env::var("PATH").unwrap();
        let separator = if cfg!(windows) { ";" } else { ":" };

        assert!(new_path.starts_with(&format!("./bin{}", separator)));
        assert!(new_path.contains(&original_path));

        // Restore original PATH
        std::env::set_var("PATH", original_path);
    }
}
