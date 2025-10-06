use std::path::{Path, PathBuf};
use std::process::{Child, Command};

use crate::errors::Result;

/// Supported shell types for command execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shell {
    /// Windows Command Prompt
    Cmd,
    /// Windows PowerShell
    PowerShell,
    /// Unix/Linux shell
    Sh,
    /// Bash shell
    Bash,
}

/// Platform-specific execution abstraction
pub trait PlatformExecutor {
    /// Spawn a process with the given command
    fn spawn_process(&self, command: &Command) -> Result<Child>;
    
    /// Normalize a path for the current platform
    fn normalize_path(&self, path: &Path) -> PathBuf;
    
    /// Get the default shell for the platform
    fn get_shell(&self) -> Shell;
    
    /// Get the PATH environment variable separator for the platform
    fn get_path_separator(&self) -> &'static str;
    
    /// Build a shell command for executing a script string
    fn build_shell_command(&self, script: &str) -> Command;
    
    /// Get available shells on this platform
    fn get_available_shells(&self) -> Vec<Shell>;
}

/// Windows platform executor
#[derive(Debug, Default)]
pub struct WindowsExecutor {
    preferred_shell: Shell,
}

impl WindowsExecutor {
    pub fn new() -> Self {
        Self {
            preferred_shell: Shell::Cmd,
        }
    }
    
    pub fn with_shell(shell: Shell) -> Self {
        Self {
            preferred_shell: shell,
        }
    }
    
    /// Check if PowerShell is available on the system
    fn is_powershell_available(&self) -> bool {
        Command::new("powershell")
            .arg("-Version")
            .output()
            .is_ok()
    }
}

impl PlatformExecutor for WindowsExecutor {
    fn spawn_process(&self, command: &Command) -> Result<Child> {
        Ok(command.spawn()?)
    }
    
    fn normalize_path(&self, path: &Path) -> PathBuf {
        // Convert forward slashes to backslashes on Windows
        let path_str = path.to_string_lossy();
        let normalized = path_str.replace('/', "\\");
        PathBuf::from(normalized)
    }
    
    fn get_shell(&self) -> Shell {
        match self.preferred_shell {
            Shell::PowerShell if self.is_powershell_available() => Shell::PowerShell,
            _ => Shell::Cmd,
        }
    }
    
    fn get_path_separator(&self) -> &'static str {
        ";"
    }
    
    fn build_shell_command(&self, script: &str) -> Command {
        match self.get_shell() {
            Shell::PowerShell => {
                let mut cmd = Command::new("powershell");
                cmd.arg("-Command").arg(script);
                cmd
            }
            Shell::Cmd => {
                let mut cmd = Command::new("cmd");
                cmd.arg("/C").arg(script);
                cmd
            }
            _ => {
                // Fallback to cmd
                let mut cmd = Command::new("cmd");
                cmd.arg("/C").arg(script);
                cmd
            }
        }
    }
    
    fn get_available_shells(&self) -> Vec<Shell> {
        let mut shells = vec![Shell::Cmd];
        if self.is_powershell_available() {
            shells.push(Shell::PowerShell);
        }
        shells
    }
}

/// Unix/Linux platform executor  
#[derive(Debug, Default)]
pub struct UnixExecutor {
    preferred_shell: Shell,
}

impl UnixExecutor {
    pub fn new() -> Self {
        Self {
            preferred_shell: Shell::Sh,
        }
    }
    
    pub fn with_shell(shell: Shell) -> Self {
        Self {
            preferred_shell: shell,
        }
    }
    
    /// Check if bash is available on the system
    fn is_bash_available(&self) -> bool {
        Command::new("bash")
            .arg("--version")
            .output()
            .is_ok()
    }
}

impl PlatformExecutor for UnixExecutor {
    fn spawn_process(&self, command: &Command) -> Result<Child> {
        Ok(command.spawn()?)
    }
    
    fn normalize_path(&self, path: &Path) -> PathBuf {
        // Unix paths are already normalized
        path.to_path_buf()
    }
    
    fn get_shell(&self) -> Shell {
        match self.preferred_shell {
            Shell::Bash if self.is_bash_available() => Shell::Bash,
            _ => Shell::Sh,
        }
    }
    
    fn get_path_separator(&self) -> &'static str {
        ":"
    }
    
    fn build_shell_command(&self, script: &str) -> Command {
        match self.get_shell() {
            Shell::Bash => {
                let mut cmd = Command::new("bash");
                cmd.arg("-c").arg(script);
                cmd
            }
            Shell::Sh => {
                let mut cmd = Command::new("sh");
                cmd.arg("-c").arg(script);
                cmd
            }
            _ => {
                // Fallback to sh
                let mut cmd = Command::new("sh");
                cmd.arg("-c").arg(script);
                cmd
            }
        }
    }
    
    fn get_available_shells(&self) -> Vec<Shell> {
        let mut shells = vec![Shell::Sh];
        if self.is_bash_available() {
            shells.push(Shell::Bash);
        }
        shells
    }
}

/// Get the appropriate platform executor for the current OS
pub fn get_platform_executor() -> Box<dyn PlatformExecutor> {
    if cfg!(target_os = "windows") {
        Box::new(WindowsExecutor::new())
    } else {
        Box::new(UnixExecutor::new())
    }
}

/// Get a platform executor with a specific shell preference
pub fn get_platform_executor_with_shell(shell: Shell) -> Box<dyn PlatformExecutor> {
    if cfg!(target_os = "windows") {
        Box::new(WindowsExecutor::with_shell(shell))
    } else {
        Box::new(UnixExecutor::with_shell(shell))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_path_normalization() {
        let executor = WindowsExecutor::new();
        let path = Path::new("src/main/java/com/example");
        let normalized = executor.normalize_path(path);
        assert_eq!(normalized.to_string_lossy(), "src\\main\\java\\com\\example");
    }

    #[test]
    fn test_unix_path_normalization() {
        let executor = UnixExecutor::new();
        let path = Path::new("src/main/java/com/example");
        let normalized = executor.normalize_path(path);
        assert_eq!(normalized, path);
    }

    #[test]
    fn test_path_separators() {
        let windows_executor = WindowsExecutor::new();
        assert_eq!(windows_executor.get_path_separator(), ";");
        
        let unix_executor = UnixExecutor::new();
        assert_eq!(unix_executor.get_path_separator(), ":");
    }

    #[test]
    fn test_shell_command_building() {
        let windows_executor = WindowsExecutor::new();
        let cmd = windows_executor.build_shell_command("echo hello");
        assert_eq!(cmd.get_program(), "cmd");
        
        let unix_executor = UnixExecutor::new();
        let cmd = unix_executor.build_shell_command("echo hello");
        assert_eq!(cmd.get_program(), "sh");
    }

    #[test]
    fn test_get_platform_executor() {
        let executor = get_platform_executor();
        // Just verify it returns something without panicking
        let _separator = executor.get_path_separator();
    }
}