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
    ) -> std::io::Result<()> {
        debug!(
            "Spawning command '{}' (prefix={}, label={:?}, color={:?})",
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
        let mut any_failed = false;

        let mut children = std::mem::take(&mut self.children);
        let len = children.len();

        for i in 0..len {
            let status = children[i].child.wait()?;
            if !status.success() {
                let code = status.code().unwrap_or(-1);
                warn!(
                    "Process '{}' failed with exit code {}",
                    children[i].name, code
                );
                any_failed = true;
                if self.fail_fast {
                    debug!("Fail-fast enabled, killing remaining processes");
                    for child in children.iter_mut().skip(i + 1) {
                        let _ = child.child.kill();
                    }
                    break;
                }
            } else {
                debug!("Process '{}' completed successfully", children[i].name);
            }
        }

        for mut child_info in children {
            if let Some(handle) = child_info.stdout_handle.take() {
                let _ = handle.join();
            }
            if let Some(handle) = child_info.stderr_handle.take() {
                let _ = handle.join();
            }
        }

        if any_failed {
            debug!("One or more processes failed");
            std::process::exit(1);
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
