// src/process.rs
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

use log::{debug, error, info, warn};
use std::{
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
    thread::{self, JoinHandle},
};

use crate::errors::BodoError;
use colored::{Color, Colorize};

pub struct ChildProcess {
    pub name: String,
    pub child: Child,
    pub stdout_handle: Option<JoinHandle<()>>,
    pub stderr_handle: Option<JoinHandle<()>>,
}

pub struct ProcessManager {
    pub children: Vec<ChildProcess>,
    pub fail_fast: bool,
}

impl ProcessManager {
    pub fn new(fail_fast: bool) -> Self {
        debug!("Creating ProcessManager with fail_fast={}", fail_fast);
        Self {
            children: Vec::new(),
            fail_fast,
        }
    }

    pub fn spawn_command(
        &mut self,
        name: &str,
        cmd: &str,
        prefix_enabled: bool,
        prefix_label: Option<String>,
        prefix_color: Option<String>,
        working_dir: Option<&str>,
    ) -> std::io::Result<()> {
        debug!(
            "Spawning command '{}' (prefix={}, label={:?}, color={:?}, working_dir={:?})",
            cmd, prefix_enabled, prefix_label, prefix_color, working_dir
        );

        let mut command = if cfg!(target_os = "windows") {
            let mut cmd_command = Command::new("cmd");
            cmd_command.arg("/C").arg(cmd);
            cmd_command
        } else {
            let mut sh_command = Command::new("sh");
            sh_command.arg("-c").arg(cmd);
            sh_command
        };

        if let Some(dir) = working_dir {
            command.current_dir(dir);
        }

        command.stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut child = command.spawn()?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let name_str = name.to_string();
        let label_str = prefix_label.clone().unwrap_or_else(|| name_str.clone());
        let color_str = prefix_color.clone();

        let stdout_handle = stdout.map(|out| {
            let label = label_str.clone();
            let color = color_str.clone();
            thread::spawn(move || {
                let reader = BufReader::new(out);
                for line in reader.lines().map_while(Result::ok) {
                    if prefix_enabled {
                        let colored_line = color_line(&label, &color, &line, false);
                        info!("{}", colored_line);
                    } else {
                        println!("{}", line);
                    }
                }
            })
        });

        let stderr_handle = stderr.map(|err| {
            let label = label_str;
            let color = color_str;
            thread::spawn(move || {
                let reader = BufReader::new(err);
                for line in reader.lines().map_while(Result::ok) {
                    if prefix_enabled {
                        let colored_line = color_line(&label, &color, &line, true);
                        error!("{}", colored_line);
                    } else {
                        eprintln!("{}", line);
                    }
                }
            })
        });

        self.children.push(ChildProcess {
            name: name_str,
            child,
            stdout_handle,
            stderr_handle,
        });

        Ok(())
    }

    pub fn run_concurrently(&mut self) -> std::io::Result<()> {
        debug!("Running {} processes concurrently", self.children.len());

        let children = std::mem::take(&mut self.children);
        let len = children.len();

        // Create a shared flag for fail-fast coordination
        let should_terminate = Arc::new(AtomicBool::new(false));

        // Create a vector to store the wait futures
        let mut wait_handles = Vec::with_capacity(len);
        let mut io_handles = Vec::with_capacity(len);

        // Move each child into its own thread
        for mut child_info in children {
            let name = child_info.name.clone();
            let stdout_handle = child_info.stdout_handle.take();
            let stderr_handle = child_info.stderr_handle.take();
            let fail_fast = self.fail_fast;
            let should_terminate = should_terminate.clone();

            let handle = thread::spawn(move || {
                // Try to wait with a timeout to allow checking the termination flag
                loop {
                    if should_terminate.load(Ordering::SeqCst) {
                        debug!("Process '{}' received termination signal", name);
                        let _ = child_info.child.kill();
                        break Ok::<(String, i32, bool), std::io::Error>((name, -1, fail_fast));
                    }

                    match child_info.child.try_wait()? {
                        Some(status) => {
                            let code = status.code().unwrap_or(-1);
                            if code != 0 && fail_fast {
                                should_terminate.store(true, Ordering::SeqCst);
                            }
                            break Ok((name, code, fail_fast));
                        }
                        None => {
                            // Process still running, sleep briefly then check again
                            thread::sleep(std::time::Duration::from_millis(10));
                        }
                    }
                }
            });

            wait_handles.push(handle);

            // Store IO handles if they exist
            if let Some(h) = stdout_handle {
                io_handles.push(h);
            }
            if let Some(h) = stderr_handle {
                io_handles.push(h);
            }
        }

        // Wait for all processes to complete
        for handle in wait_handles {
            match handle.join().unwrap() {
                Ok((name, code, _)) => {
                    if code != 0 {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Process '{}' failed with exit code {}", name, code),
                        ));
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        // Wait for all IO threads to complete
        for handle in io_handles {
            let _ = handle.join();
        }

        Ok(())
    }

    pub fn kill_all(&mut self) -> Result<(), BodoError> {
        warn!("kill_all called, best effort kill all children...");
        let mut children = std::mem::take(&mut self.children);
        for child in &mut children {
            let _ = child.child.kill();
        }
        self.children = children;
        Ok(())
    }
}

fn color_line(prefix: &str, prefix_color: &Option<String>, line: &str, is_stderr: bool) -> String {
    let default_color = if is_stderr { Color::Red } else { Color::White };
    let color = prefix_color
        .as_ref()
        .and_then(|c| parse_color(c))
        .unwrap_or(default_color);
    let colored_prefix = format!("[{}]", prefix).color(color);
    format!("{} {}", colored_prefix, line)
}

fn parse_color(c: &str) -> Option<Color> {
    debug!("Parsing color: {}", c);
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
        _ => {
            debug!("Unknown color: {}", c);
            None
        }
    }
}

#[cfg(test)]
pub use parse_color;

// End of src/process.rs
