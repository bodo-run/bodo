use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

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
                let value = value.trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                
                // Only set if not already in environment
                if std::env::var(&key).is_err() {
                    self.env_vars.insert(key.clone(), value.clone());
                    std::env::set_var(key, value);
                }
            }
        }
        Ok(())
    }

    pub fn inject_exec_paths(&mut self, exec_paths: &[String]) {
        if let Ok(current_path) = std::env::var("PATH") {
            let new_paths: Vec<String> = exec_paths.iter()
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
            let new_path = format!("{}{}{}", 
                new_paths.join(path_separator),
                path_separator,
                current_path
            );
            
            std::env::set_var("PATH", new_path);
        }
    }

    pub fn get_env(&self) -> &HashMap<String, String> {
        &self.env_vars
    }
} 