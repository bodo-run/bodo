//! Sandbox module for safe command execution in dry-run mode
//!
//! This module provides sandboxing capabilities using bubblewrap (bwrap) or firejail
//! to execute commands in an isolated environment where filesystem changes can be
//! monitored without affecting the actual system.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

use crate::errors::{BodoError, Result};
use crate::plugin::SideEffect;

/// Sandbox implementation for safe command execution
pub struct Sandbox {
    /// Temporary directory for sandbox operations
    #[allow(dead_code)]
    temp_dir: TempDir,
    /// Path to the sandbox root
    sandbox_root: PathBuf,
    /// Whether bubblewrap is available
    has_bwrap: bool,
    /// Whether firejail is available
    has_firejail: bool,
}

impl Sandbox {
    /// Create a new sandbox instance
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new()
            .map_err(|e| BodoError::PluginError(format!("Failed to create temp dir: {}", e)))?;

        let sandbox_root = temp_dir.path().to_path_buf();

        // Check for available sandboxing tools
        let has_bwrap = Command::new("which")
            .arg("bwrap")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let has_firejail = Command::new("which")
            .arg("firejail")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        Ok(Self {
            temp_dir,
            sandbox_root,
            has_bwrap,
            has_firejail,
        })
    }

    /// Execute a command in the sandbox and analyze side effects
    pub fn execute_and_analyze(
        &self,
        command: &str,
        working_dir: &Path,
        env: &std::collections::HashMap<String, String>,
    ) -> Result<Vec<SideEffect>> {
        // Create sandbox directories
        self.setup_sandbox_dirs(working_dir)?;

        // Take snapshot of sandbox before execution
        let before_snapshot = self.take_filesystem_snapshot()?;

        // Execute command in sandbox
        let execution_result = if self.has_bwrap {
            self.execute_with_bwrap(command, working_dir, env)?
        } else if self.has_firejail {
            self.execute_with_firejail(command, working_dir, env)?
        } else {
            // Fallback to restricted execution without containerization
            self.execute_with_restrictions(command, working_dir, env)?
        };

        // Take snapshot after execution
        let after_snapshot = self.take_filesystem_snapshot()?;

        // Analyze differences
        let mut side_effects =
            self.analyze_filesystem_changes(&before_snapshot, &after_snapshot)?;

        // Add process spawn side effect
        side_effects.push(SideEffect::ProcessSpawn(command.to_string()));

        // Analyze command output for additional side effects
        if let Some(additional) = self.analyze_command_output(&execution_result) {
            side_effects.extend(additional);
        }

        Ok(side_effects)
    }

    /// Setup sandbox directory structure
    fn setup_sandbox_dirs(&self, working_dir: &Path) -> Result<()> {
        // Create basic directory structure
        let dirs = ["tmp", "home", "work"];
        for dir in &dirs {
            fs::create_dir_all(self.sandbox_root.join(dir)).map_err(|e| {
                BodoError::PluginError(format!("Failed to create sandbox dir: {}", e))
            })?;
        }

        // Create working directory in sandbox
        if working_dir.is_absolute() {
            let relative_path = working_dir.strip_prefix("/").unwrap_or(working_dir);
            let sandbox_work_dir = self.sandbox_root.join("work").join(relative_path);
            fs::create_dir_all(&sandbox_work_dir)
                .map_err(|e| BodoError::PluginError(format!("Failed to create work dir: {}", e)))?;
        }

        Ok(())
    }

    /// Execute command using bubblewrap
    fn execute_with_bwrap(
        &self,
        command: &str,
        working_dir: &Path,
        env: &std::collections::HashMap<String, String>,
    ) -> Result<CommandOutput> {
        let mut cmd = Command::new("bwrap");

        // Basic isolation flags
        cmd.arg("--unshare-all")
            .arg("--share-net") // Allow network for analysis but could be disabled
            .arg("--die-with-parent")
            .arg("--new-session");

        // Mount points
        cmd.arg("--ro-bind")
            .arg("/usr")
            .arg("/usr")
            .arg("--ro-bind")
            .arg("/bin")
            .arg("/bin")
            .arg("--ro-bind")
            .arg("/lib")
            .arg("/lib")
            .arg("--ro-bind")
            .arg("/lib64")
            .arg("/lib64")
            .arg("--ro-bind")
            .arg("/etc/resolv.conf")
            .arg("/etc/resolv.conf");

        // Sandbox directories
        cmd.arg("--bind")
            .arg(self.sandbox_root.join("tmp"))
            .arg("/tmp")
            .arg("--bind")
            .arg(self.sandbox_root.join("home"))
            .arg("/home")
            .arg("--bind")
            .arg(self.sandbox_root.join("work"))
            .arg("/work");

        // Set working directory
        let sandbox_cwd = if working_dir.is_absolute() {
            PathBuf::from("/work").join(working_dir.strip_prefix("/").unwrap_or(working_dir))
        } else {
            PathBuf::from("/work").join(working_dir)
        };
        cmd.arg("--chdir").arg(&sandbox_cwd);

        // Environment variables
        for (key, value) in env {
            cmd.arg("--setenv").arg(key).arg(value);
        }

        // Set safe environment defaults
        cmd.arg("--setenv")
            .arg("HOME")
            .arg("/home")
            .arg("--setenv")
            .arg("TMPDIR")
            .arg("/tmp")
            .arg("--setenv")
            .arg("PATH")
            .arg("/usr/bin:/bin");

        // Execute the actual command
        cmd.arg("/bin/sh").arg("-c").arg(command);

        let output = cmd
            .output()
            .map_err(|e| BodoError::PluginError(format!("Failed to execute in bwrap: {}", e)))?;

        Ok(CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
        })
    }

    /// Execute command using firejail
    fn execute_with_firejail(
        &self,
        command: &str,
        working_dir: &Path,
        env: &std::collections::HashMap<String, String>,
    ) -> Result<CommandOutput> {
        let mut cmd = Command::new("firejail");

        // Basic isolation
        cmd.arg("--quiet")
            .arg(format!(
                "--private={}",
                self.sandbox_root.join("home").display()
            ))
            .arg("--private-tmp")
            .arg("--private-dev")
            .arg("--nosound")
            .arg("--no3d");

        // Network isolation (optional)
        // cmd.arg("--net=none");

        // Set working directory
        if working_dir.is_absolute() {
            cmd.current_dir(working_dir);
        }

        // Environment variables
        for (key, value) in env {
            cmd.env(key, value);
        }

        // Execute command
        cmd.arg("--").arg("/bin/sh").arg("-c").arg(command);

        let output = cmd
            .output()
            .map_err(|e| BodoError::PluginError(format!("Failed to execute in firejail: {}", e)))?;

        Ok(CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
        })
    }

    /// Fallback execution with basic restrictions (when no sandbox tool is available)
    fn execute_with_restrictions(
        &self,
        command: &str,
        _working_dir: &Path,
        env: &std::collections::HashMap<String, String>,
    ) -> Result<CommandOutput> {
        // Create a restricted environment
        let mut cmd = Command::new("/bin/sh");
        cmd.arg("-c").arg(command);

        // Change to sandbox directory
        let sandbox_work = self.sandbox_root.join("work");
        cmd.current_dir(&sandbox_work);

        // Set restricted environment
        cmd.env_clear();
        cmd.env("HOME", self.sandbox_root.join("home"))
            .env("TMPDIR", self.sandbox_root.join("tmp"))
            .env("PATH", "/usr/bin:/bin");

        // Add user-specified environment variables
        for (key, value) in env {
            cmd.env(key, value);
        }

        // Note: This is less secure than bwrap/firejail but better than nothing
        // Real filesystem operations will still occur but in the temp directory

        let output = cmd
            .output()
            .map_err(|e| BodoError::PluginError(format!("Failed to execute command: {}", e)))?;

        Ok(CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
        })
    }

    /// Take a snapshot of the filesystem state
    fn take_filesystem_snapshot(&self) -> Result<FilesystemSnapshot> {
        let mut files = HashSet::new();
        let mut directories = HashSet::new();

        self.scan_directory(&self.sandbox_root, &mut files, &mut directories)?;

        Ok(FilesystemSnapshot { files, directories })
    }

    /// Recursively scan directory for files and subdirectories
    #[allow(clippy::only_used_in_recursion)]
    fn scan_directory(
        &self,
        dir: &Path,
        files: &mut HashSet<PathBuf>,
        directories: &mut HashSet<PathBuf>,
    ) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        let entries = fs::read_dir(dir)
            .map_err(|e| BodoError::PluginError(format!("Failed to read directory: {}", e)))?;

        for entry in entries {
            let entry = entry
                .map_err(|e| BodoError::PluginError(format!("Failed to read entry: {}", e)))?;
            let path = entry.path();

            if path.is_dir() {
                directories.insert(path.clone());
                self.scan_directory(&path, files, directories)?;
            } else {
                files.insert(path);
            }
        }

        Ok(())
    }

    /// Analyze filesystem changes between snapshots
    fn analyze_filesystem_changes(
        &self,
        before: &FilesystemSnapshot,
        after: &FilesystemSnapshot,
    ) -> Result<Vec<SideEffect>> {
        let mut side_effects = Vec::new();

        // Find new files (writes)
        for file in &after.files {
            if !before.files.contains(file) {
                // Convert sandbox path to relative path
                if let Ok(relative) = file.strip_prefix(&self.sandbox_root) {
                    side_effects.push(SideEffect::FileWrite(PathBuf::from("/").join(relative)));
                }
            }
        }

        // Find deleted files (could be tracked as a different side effect type)
        for file in &before.files {
            if !after.files.contains(file) {
                // Could add a FileDelete side effect type if needed
                if let Ok(relative) = file.strip_prefix(&self.sandbox_root) {
                    // For now, we'll note it as a write operation (modification)
                    side_effects.push(SideEffect::FileWrite(PathBuf::from("/").join(relative)));
                }
            }
        }

        // Find modified files (simplified: checking if content changed would require checksums)
        // This is a simplified version; a full implementation would compare file contents/timestamps

        Ok(side_effects)
    }

    /// Analyze command output for additional side effects
    fn analyze_command_output(&self, output: &CommandOutput) -> Option<Vec<SideEffect>> {
        let mut side_effects = Vec::new();

        // Check for network operations in output
        let network_patterns = ["curl", "wget", "http://", "https://", "ftp://"];
        let combined_output = format!("{}\n{}", output.stdout, output.stderr);

        for pattern in &network_patterns {
            if combined_output.contains(pattern) {
                // Extract URL if possible
                for line in combined_output.lines() {
                    if line.contains(pattern) {
                        side_effects.push(SideEffect::NetworkRequest(line.to_string()));
                        break;
                    }
                }
            }
        }

        if side_effects.is_empty() {
            None
        } else {
            Some(side_effects)
        }
    }
}

/// Represents the output of a command execution
struct CommandOutput {
    stdout: String,
    stderr: String,
    #[allow(dead_code)]
    exit_code: i32,
}

/// Snapshot of filesystem state
struct FilesystemSnapshot {
    files: HashSet<PathBuf>,
    #[allow(dead_code)]
    directories: HashSet<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_creation() {
        let sandbox = Sandbox::new();
        assert!(sandbox.is_ok());

        let sandbox = sandbox.unwrap();
        assert!(sandbox.sandbox_root.exists());
    }

    #[test]
    fn test_sandbox_directory_setup() {
        let sandbox = Sandbox::new().unwrap();
        let working_dir = Path::new("/test/dir");

        let result = sandbox.setup_sandbox_dirs(working_dir);
        assert!(result.is_ok());

        // Check that sandbox directories were created
        assert!(sandbox.sandbox_root.join("tmp").exists());
        assert!(sandbox.sandbox_root.join("home").exists());
        assert!(sandbox.sandbox_root.join("work").exists());
    }
}
