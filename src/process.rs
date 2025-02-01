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
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::thread;

use crate::errors::BodoError;
use colored::{Color, Colorize};

pub struct ProcessManager {
    children: Vec<(String, Child)>,
    fail_fast: bool,
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
    ) -> std::io::Result<()> {
        debug!(
            "Spawning command '{}' with prefix_enabled={}, label={:?}, color={:?}",
            cmd, prefix_enabled, prefix_label, prefix_color
        );

        let mut child = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let mc_name = name.to_string();

        let stdout_handle = stdout.map(|stdout| {
            let prefix_label = prefix_label.clone();
            let prefix_color = prefix_color.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    if prefix_enabled {
                        let prefix_str = prefix_label.as_deref().unwrap_or("");
                        let colorized = color_line(prefix_str, &prefix_color, &line, false);
                        info!("{}", colorized);
                    } else {
                        println!("{}", line);
                    }
                }
            })
        });

        let stderr_handle = stderr.map(|stderr| {
            let prefix_label = prefix_label.clone();
            let prefix_color = prefix_color.clone();
            let mc_name = mc_name.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    if prefix_enabled {
                        let prefix_str = prefix_label.as_deref().unwrap_or(&mc_name);
                        let colorized = color_line(prefix_str, &prefix_color, &line, true);
                        error!("{}", colorized);
                    } else {
                        eprintln!("{}", line);
                    }
                }
            })
        });

        match child.wait() {
            Ok(status) => {
                if !status.success() {
                    let code = status.code().unwrap_or(-1);
                    warn!("'{}' failed with exit code {}", mc_name, code);
                } else {
                    debug!("'{}' completed successfully", mc_name);
                }
            }
            Err(e) => {
                warn!("Error waiting on '{}': {}", mc_name, e);
            }
        }

        if let Some(handle) = stdout_handle {
            let _ = handle.join();
        }
        if let Some(handle) = stderr_handle {
            let _ = handle.join();
        }

        Ok(())
    }

    pub fn run_concurrently(&mut self) -> std::io::Result<()> {
        debug!("Running {} processes concurrently", self.children.len());
        let mut any_failed = false;

        for (name, mut child) in self.children.drain(..) {
            match child.wait() {
                Ok(status) => {
                    if !status.success() {
                        let code = status.code().unwrap_or(-1);
                        warn!("'{}' failed with exit code {}", name, code);
                        any_failed = true;
                        if self.fail_fast {
                            debug!("Fail-fast enabled, stopping remaining processes");
                            break;
                        }
                    } else {
                        debug!("'{}' completed successfully", name);
                    }
                }
                Err(e) => {
                    warn!("Error waiting on '{}': {}", name, e);
                    any_failed = true;
                    if self.fail_fast {
                        debug!("Fail-fast enabled, stopping remaining processes");
                        break;
                    }
                }
            }
        }

        if any_failed {
            debug!("One or more processes failed");
            std::process::exit(1);
        }

        Ok(())
    }

    pub fn kill_all(&self) -> Result<(), BodoError> {
        warn!("kill_all called but is now a no-op.");
        Ok(())
    }
}

fn color_line(
    prefix_label: &str,
    prefix_color: &Option<String>,
    line: &str,
    is_stderr: bool,
) -> String {
    let default_color = if is_stderr { Color::Red } else { Color::White };

    let color = prefix_color
        .as_ref()
        .and_then(|s| parse_color(s))
        .unwrap_or(default_color);

    let colored_prefix = format!("[{}]", prefix_label).color(color);
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
