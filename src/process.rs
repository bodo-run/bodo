use std::error::Error;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, RwLock,
};
use std::thread::{self, JoinHandle};

use crate::errors::BodoError;
use crate::task::ColorSpec;
use colored::Colorize;

/// Holds a single child process plus some metadata.
struct ManagedChild {
    name: String,
    child: Child,
    color: Option<ColorSpec>,
}

/// A "fail fast" process manager that can spawn concurrent processes
/// and kill them if any child fails (if `fail_fast` is true).
pub struct ProcessManager {
    children: Arc<Mutex<Vec<ManagedChild>>>,
    fail_fast: bool,
    any_failure: Arc<RwLock<Option<String>>>,
    stop_signal: Arc<AtomicBool>,
    threads: Vec<JoinHandle<()>>,
}

impl ProcessManager {
    /// Create a new manager. If `fail_fast` is true, the first failure
    /// triggers killing all other processes immediately.
    pub fn new(fail_fast: bool) -> Self {
        Self {
            children: Arc::new(Mutex::new(Vec::new())),
            fail_fast,
            any_failure: Arc::new(RwLock::new(None)),
            stop_signal: Arc::new(AtomicBool::new(false)),
            threads: Vec::new(),
        }
    }

    /// Spawn a command using the system shell. The `name` is just a label
    /// that helps identify the process in logs/errors.
    pub fn spawn_command(
        &mut self,
        name: &str,
        command: &str,
        color: Option<ColorSpec>,
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
            child,
            color,
        };

        // Store it in the shared list
        self.children.lock().unwrap().push(managed_child);

        Ok(())
    }

    /// After spawning commands, call this to actually run them concurrently.
    /// Returns Ok if all processes succeed (exit code 0), or Err if any fail.
    pub fn run_concurrently(&mut self) -> Result<(), BodoError> {
        let children_for_threads = Arc::clone(&self.children);
        let any_failure_for_threads = Arc::clone(&self.any_failure);
        let stop_for_threads = Arc::clone(&self.stop_signal);
        let fail_fast = self.fail_fast;

        // Create a waiter thread for each child
        {
            let mut locked = children_for_threads.lock().unwrap();
            for idx in 0..locked.len() {
                let mc_name = locked[idx].name.clone();
                let mc_color = locked[idx].color.clone();
                let mut child_orig = std::mem::replace(&mut locked[idx].child, dummy_child()?);

                let c_any_failure = Arc::clone(&any_failure_for_threads);
                let c_stop_signal = Arc::clone(&stop_for_threads);
                let c_children = Arc::clone(&children_for_threads);

                // Create stdout/stderr threads
                if let Some(stdout) = child_orig.stdout.take() {
                    let name = mc_name.clone();
                    let color = mc_color.clone();
                    let stop_signal = Arc::clone(&c_stop_signal);
                    thread::spawn(move || {
                        let reader = BufReader::new(stdout);
                        for line in reader.lines() {
                            // Check if we should stop
                            if stop_signal.load(Ordering::SeqCst) {
                                break;
                            }
                            if let Ok(line) = line {
                                // Don't print if stop signal is set
                                if !stop_signal.load(Ordering::SeqCst) {
                                    let prefix = format!("[{}]", name);
                                    let prefix_colored = apply_color(&prefix, color.as_ref());
                                    println!("{} {}", prefix_colored, line);
                                }
                            }
                        }
                    });
                }

                if let Some(stderr) = child_orig.stderr.take() {
                    let name = mc_name.clone();
                    let color = mc_color.clone();
                    let stop_signal = Arc::clone(&c_stop_signal);
                    thread::spawn(move || {
                        let reader = BufReader::new(stderr);
                        for line in reader.lines() {
                            // Check if we should stop
                            if stop_signal.load(Ordering::SeqCst) {
                                break;
                            }
                            if let Ok(line) = line {
                                // Don't print if stop signal is set
                                if !stop_signal.load(Ordering::SeqCst) {
                                    let prefix = format!("[{}]", name);
                                    let prefix_colored = apply_color(&prefix, color.as_ref());
                                    eprintln!("{} {}", prefix_colored, line);
                                }
                            }
                        }
                    });
                }

                // Create a dedicated waiter thread that handles both waiting and killing
                let handle = thread::spawn(move || {
                    let status_res = child_orig.wait();
                    match status_res {
                        Ok(status) => {
                            if !status.success() {
                                let mut w = c_any_failure.write().unwrap();
                                if w.is_none() {
                                    *w = Some(format!(
                                        "'{}' failed with code {:?}",
                                        mc_name,
                                        status.code().unwrap_or(1)
                                    ));
                                }
                                if fail_fast {
                                    // Signal stop immediately
                                    c_stop_signal.store(true, Ordering::SeqCst);
                                    // Kill all other processes
                                    let mut locked_ch = c_children.lock().unwrap();
                                    for c in locked_ch.iter_mut() {
                                        let _ = kill_child(&mut c.child);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            let mut w = c_any_failure.write().unwrap();
                            if w.is_none() {
                                *w = Some(format!("Error waiting on '{}': {}", mc_name, e));
                            }
                            if fail_fast {
                                // Signal stop immediately
                                c_stop_signal.store(true, Ordering::SeqCst);
                                // Kill all other processes
                                let mut locked_ch = c_children.lock().unwrap();
                                for c in locked_ch.iter_mut() {
                                    let _ = kill_child(&mut c.child);
                                }
                            }
                        }
                    }
                });

                self.threads.push(handle);
            }
        }

        // Just wait for all threads to finish
        for handle in self.threads.drain(..) {
            let _ = handle.join(); // ignore panics
        }

        // If we have an error, return it
        let error_opt = &*self.any_failure.read().unwrap();
        if let Some(msg) = error_opt {
            return Err(BodoError::PluginError(msg.clone()));
        }

        Ok(())
    }

    /// Kills all running processes (if any).
    pub fn kill_all(&self) -> Result<(), BodoError> {
        self.stop_signal.store(true, Ordering::SeqCst);
        let mut locked = self.children.lock().unwrap();
        for mc in locked.iter_mut() {
            kill_child(&mut mc.child)?;
        }
        Ok(())
    }
}

/// Hack: You can't trivially build a "dummy" child in stable Rust,
/// but we need to fill in the vector so we can move the real Child into a thread.
fn dummy_child() -> Result<Child, BodoError> {
    let child = Command::new("sh")
        .arg("-c")
        .arg("echo dummy_child")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| BodoError::PluginError(format!("Failed to create dummy child: {}", e)))?;
    Ok(child)
}

/// Attempt to kill a child gracefully, then forcibly if needed.
fn kill_child(child: &mut Child) -> Result<(), BodoError> {
    #[cfg(unix)]
    {
        // First try Child::kill, which sends SIGKILL
        let _ = child.kill();
        let _ = child.wait();

        // Kill the process group to ensure all children are killed
        unsafe {
            let pid = child.id() as libc::pid_t;
            let _ = libc::kill(-pid, libc::SIGKILL); // negative pid means kill process group
            let _ = libc::kill(pid, libc::SIGKILL); // also try direct kill
        }
    }

    #[cfg(not(unix))]
    {
        let _ = child.kill();
        let _ = child.wait();
    }

    Ok(())
}

fn apply_color(text: &str, color: Option<&ColorSpec>) -> colored::ColoredString {
    if let Some(color) = color {
        match color {
            ColorSpec::Black => text.black(),
            ColorSpec::Red => text.red(),
            ColorSpec::Green => text.green(),
            ColorSpec::Yellow => text.yellow(),
            ColorSpec::Blue => text.blue(),
            ColorSpec::Magenta => text.magenta(),
            ColorSpec::Cyan => text.cyan(),
            ColorSpec::White => text.white(),
            ColorSpec::BrightBlack => text.bright_black(),
            ColorSpec::BrightRed => text.bright_red(),
            ColorSpec::BrightGreen => text.bright_green(),
            ColorSpec::BrightYellow => text.bright_yellow(),
            ColorSpec::BrightBlue => text.bright_blue(),
            ColorSpec::BrightMagenta => text.bright_magenta(),
            ColorSpec::BrightCyan => text.bright_cyan(),
            ColorSpec::BrightWhite => text.bright_white(),
        }
    } else {
        text.normal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concurrent_run() {
        let mut pm = ProcessManager::new(true);
        pm.spawn_command("test1", "echo test1", None).unwrap();
        pm.spawn_command("test2", "echo test2", None).unwrap();
        pm.run_concurrently().unwrap();
    }
}
