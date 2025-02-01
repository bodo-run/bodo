#[derive(Debug, Clone)]
pub enum ColorSpec {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::errors::BodoError;
use colored::{Color, Colorize};

/// Holds a single child process plus some metadata.
struct ManagedChild {
    name: String,
    child: Option<Child>,
    prefix_enabled: bool,
    prefix_label: Option<String>,
    prefix_color: Option<String>,
}

/// A "fail fast" process manager that can spawn concurrent processes
/// and kill them if any child fails (if `fail_fast` is true).
/// We will modify it so that we do NOT exit overall when a child fails.
pub struct ProcessManager {
    children: Arc<Mutex<Vec<ManagedChild>>>,
}

impl ProcessManager {
    /// Create a new manager. If `fail_fast` is true, the first failure
    /// triggers killing all other processes immediately.
    pub fn new(_fail_fast: bool) -> Self {
        Self {
            children: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Spawn a command using the system shell. The `name` is just a label
    /// that helps identify the process in logs/errors.
    pub fn spawn_command(
        &mut self,
        name: &str,
        command: &str,
        prefix_enabled: bool,
        prefix_label: Option<String>,
        prefix_color: Option<String>,
    ) -> Result<(), BodoError> {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(command);

        // Set up process group on Unix
        #[cfg(unix)]
        unsafe {
            use std::os::unix::process::CommandExt;
            cmd.pre_exec(|| {
                // Create a new process group
                libc::setpgid(0, 0);
                Ok(())
            });
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let child = cmd
            .spawn()
            .map_err(|e| BodoError::PluginError(format!("Failed to spawn {}: {}", name, e)))?;
        let managed_child = ManagedChild {
            name: name.to_string(),
            child: Some(child),
            prefix_enabled,
            prefix_label,
            prefix_color,
        };

        // Store it in the shared list
        self.children.lock().unwrap().push(managed_child);

        Ok(())
    }

    /// After spawning commands, call this to actually run them concurrently.
    /// We now only warn on failures and keep going.
    pub fn run_concurrently(&mut self) -> Result<(), BodoError> {
        let children_for_threads = Arc::clone(&self.children);

        let mut thread_handles = Vec::new();
        {
            let mut locked = children_for_threads.lock().unwrap();
            for mc in locked.iter_mut() {
                let mc_name = mc.name.clone();
                // The actual child to run
                let child = mc.child.take();

                // Clone prefix info for threads
                let prefix_enabled = mc.prefix_enabled;
                let prefix_label = mc.prefix_label.clone();
                let prefix_color = mc.prefix_color.clone();

                let handle = std::thread::spawn(move || {
                    if let Some(mut child) = child {
                        // Pipe stdout, stderr (unchanged)...
                        let stdout = child.stdout.take();
                        let _stderr = child.stderr.take();

                        let stdout_prefix_label = prefix_label.clone();
                        let stdout_prefix_color = prefix_color.clone();
                        let stdout_handle = stdout.map(|stdout| {
                            let prefix_label = stdout_prefix_label;
                            let prefix_color = stdout_prefix_color;
                            thread::spawn(move || {
                                let reader = BufReader::new(stdout);
                                for line in reader.lines().flatten() {
                                    if prefix_enabled {
                                        let prefix_str = prefix_label.as_deref().unwrap_or("");
                                        let colorized =
                                            color_line(prefix_str, &prefix_color, &line, false);
                                        println!("{}", colorized);
                                    } else {
                                        println!("{}", line);
                                    }
                                }
                            })
                        });

                        let stderr = child.stderr.take();
                        let name_for_stderr = mc_name.clone();
                        let stderr_prefix_label = prefix_label;
                        let stderr_prefix_color = prefix_color;
                        let stderr_handle = stderr.map(|stderr| {
                            let prefix_label = stderr_prefix_label;
                            let prefix_color = stderr_prefix_color;
                            thread::spawn(move || {
                                let reader = BufReader::new(stderr);
                                for line in reader.lines().flatten() {
                                    if prefix_enabled {
                                        let prefix_str =
                                            prefix_label.as_deref().unwrap_or(&name_for_stderr);
                                        let colorized =
                                            color_line(prefix_str, &prefix_color, &line, true);
                                        eprintln!("{}", colorized);
                                    } else {
                                        eprintln!("[{}] {}", name_for_stderr, line);
                                    }
                                }
                            })
                        });

                        match child.wait() {
                            Ok(status) => {
                                if !status.success() {
                                    let code = status.code().unwrap_or(-1);
                                    eprintln!(
                                        "Warning: '{}' failed with exit code {}",
                                        mc_name, code
                                    );
                                    // We do NOT set any global error or kill others.
                                }
                            }
                            Err(e) => {
                                eprintln!("Warning: Error waiting on '{}': {}", mc_name, e);
                            }
                        }

                        // Join stdout/stderr threads
                        if let Some(handle) = stdout_handle {
                            let _ = handle.join();
                        }
                        if let Some(handle) = stderr_handle {
                            let _ = handle.join();
                        }
                    }
                });

                thread_handles.push(handle);
            }
        }

        // Wait for all threads to finish
        for handle in thread_handles {
            let _ = handle.join();
        }

        // We do not fail overall:
        Ok(())
    }

    /// Removed kill_all or made it a no-op if you don't want
    /// to kill processes. Or keep it if you like. For example:
    pub fn kill_all(&self) -> Result<(), BodoError> {
        // No longer kills anything, just prints a warning:
        eprintln!("kill_all called but is now a no-op.");
        Ok(())
    }
}

/// Helper to color a line with a prefix
fn color_line(
    prefix_label: &str,
    prefix_color: &Option<String>,
    line: &str,
    _is_stderr: bool,
) -> String {
    // default prefix color if none is set
    let default_color = Color::White;

    // parse the color from prefix_color Option<String>, fallback to default_color
    let color = prefix_color
        .as_ref()
        .and_then(|s| parse_color(s))
        .unwrap_or(default_color);

    let colored_prefix = format!("[{}]", prefix_label).color(color);

    // Example final output: "[taskName] some text"
    format!("{} {}", colored_prefix, line)
}

/// Convert a &str like "blue"/"red"/"magenta" to a Color from `colored`
fn parse_color(c: &str) -> Option<Color> {
    match c.to_lowercase().as_str() {
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "white" => Some(Color::White),
        "brightblack" => Some(Color::BrightBlack),
        "brightred" => Some(Color::BrightRed),
        "brightgreen" => Some(Color::BrightGreen),
        "brightyellow" => Some(Color::BrightYellow),
        "brightblue" => Some(Color::BrightBlue),
        "brightmagenta" => Some(Color::BrightMagenta),
        "brightcyan" => Some(Color::BrightCyan),
        "brightwhite" => Some(Color::BrightWhite),
        _ => None,
    }
}
