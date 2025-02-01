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
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, RwLock,
};
use std::thread;

use crate::errors::BodoError;

/// Holds a single child process plus some metadata.
struct ManagedChild {
    name: String,
    child: Option<Child>,
}

/// A "fail fast" process manager that can spawn concurrent processes
/// and kill them if any child fails (if `fail_fast` is true).
pub struct ProcessManager {
    children: Arc<Mutex<Vec<ManagedChild>>>,
    fail_fast: bool,
    any_failure: Arc<RwLock<Option<String>>>,
    stop_signal: Arc<AtomicBool>,
    // threads: Vec<JoinHandle<()>>,
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
            // threads: Vec::new(),
        }
    }

    /// Spawn a command using the system shell. The `name` is just a label
    /// that helps identify the process in logs/errors.
    pub fn spawn_command(&mut self, name: &str, command: &str) -> Result<(), BodoError> {
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

        // Collect thread handles so we can join later
        let mut thread_handles = Vec::new();

        {
            let mut locked = children_for_threads.lock().unwrap();
            for mc in locked.iter_mut() {
                let mc_name = mc.name.clone();
                let child = mc.child.take(); // Use take() instead of clone()
                let c_any_failure = Arc::clone(&any_failure_for_threads);
                let c_stop_signal = Arc::clone(&stop_for_threads);
                let c_children = Arc::clone(&children_for_threads);

                // Spawn a thread to wait on the child and handle its output
                let handle = std::thread::spawn(move || {
                    if let Some(mut child) = child {
                        // Create stdout reader thread
                        let stdout = child.stdout.take();
                        let stdout_handle = stdout.map(|stdout| {
                            thread::spawn(move || {
                                let reader = BufReader::new(stdout);
                                for line in reader.lines() {
                                    if let Ok(line) = line {
                                        println!("{}", line);
                                    }
                                }
                            })
                        });

                        // Create stderr reader thread
                        let stderr = child.stderr.take();
                        let name_for_stderr = mc_name.clone();
                        let stderr_handle = stderr.map(|stderr| {
                            thread::spawn(move || {
                                let reader = BufReader::new(stderr);
                                for line in reader.lines() {
                                    if let Ok(line) = line {
                                        eprintln!("[{}] {}", name_for_stderr, line);
                                    }
                                }
                            })
                        });

                        // Wait for the child process to finish
                        match child.wait() {
                            Ok(status) => {
                                if !status.success() {
                                    // Non‐zero exit → mark failure
                                    let code = status.code().unwrap_or(-1);
                                    let mut w = c_any_failure.write().unwrap();
                                    if w.is_none() {
                                        *w = Some(format!(
                                            "'{}' failed with exit code {}",
                                            mc_name, code
                                        ));
                                    }
                                    // If fail_fast, kill all other processes
                                    if fail_fast {
                                        c_stop_signal.store(true, Ordering::SeqCst);
                                        let mut locked_ch = c_children.lock().unwrap();
                                        for c in locked_ch.iter_mut() {
                                            if let Some(child2) = &mut c.child {
                                                let _ = kill_child(child2);
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                // OS‐level error waiting on the process
                                let mut w = c_any_failure.write().unwrap();
                                if w.is_none() {
                                    *w = Some(format!("Error waiting on '{}': {}", mc_name, e));
                                }
                                if fail_fast {
                                    c_stop_signal.store(true, Ordering::SeqCst);
                                    let mut locked_ch = c_children.lock().unwrap();
                                    for c in locked_ch.iter_mut() {
                                        if let Some(child2) = &mut c.child {
                                            let _ = kill_child(child2);
                                        }
                                    }
                                }
                            }
                        }

                        // Wait for output reader threads to finish
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

        // If any thread set any_failure, return an error
        let error_opt = &*any_failure_for_threads.read().unwrap();
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
            if let Some(child) = &mut mc.child {
                kill_child(child)?;
            }
        }
        Ok(())
    }
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
