use crate::config::ColorSpec;
use crate::plugin::BodoPlugin;
use colored::Colorize;
use dialoguer::console::Term;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// Helper function to determine if color should be enabled based on config hierarchy
#[allow(dead_code)]
fn is_color_enabled(
    global_config: &Option<bool>,
    script_config: &Option<bool>,
    task_config: &Option<bool>,
    output_config: &Option<bool>,
) -> bool {
    // Check configs in order of precedence (highest to lowest)
    // If any level explicitly disables color, return false
    if output_config.unwrap_or(false)
        || task_config.unwrap_or(false)
        || script_config.unwrap_or(false)
        || global_config.unwrap_or(false)
    {
        return false;
    }
    true
}

static MAX_LABEL_WIDTH: AtomicUsize = AtomicUsize::new(0);
static HEADER_PRINTED: AtomicBool = AtomicBool::new(false);
static COMMAND_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone)]
pub struct PrintCommandPlugin;

impl PrintCommandPlugin {
    pub fn new() -> Self {
        Self
    }

    #[allow(dead_code)]
    fn get_max_width() -> usize {
        let term = Term::stdout();
        ((term.size().1 as f64) * 0.6) as usize
    }

    pub fn get_padding_width(
        concurrent_items: &[crate::config::ConcurrentItem],
        group: &str,
    ) -> usize {
        let mut max_len = 0;
        let mut command_number = 1;

        for item in concurrent_items {
            let label = match item {
                crate::config::ConcurrentItem::Task { task, output } => {
                    // If output prefix is specified, use it; otherwise fallback
                    if let Some(o) = output {
                        if let Some(prefix) = &o.prefix {
                            format!("[{}]", prefix)
                        } else {
                            format!("[{}:{}]", group, task)
                        }
                    } else {
                        format!("[{}:{}]", group, task)
                    }
                }
                crate::config::ConcurrentItem::Command {
                    command: _,
                    name,
                    output,
                } => {
                    let label_name = if let Some(n) = name {
                        n.to_string()
                    } else {
                        let auto_label = format!("command{}", command_number);
                        command_number += 1;
                        auto_label
                    };

                    // If output prefix is specified, use it; otherwise fallback
                    if let Some(o) = output {
                        if let Some(prefix) = &o.prefix {
                            format!("[{}]", prefix)
                        } else {
                            format!("[{}:{}]", group, label_name)
                        }
                    } else {
                        format!("[{}:{}]", group, label_name)
                    }
                }
            };

            max_len = max_len.max(label.len());
        }

        let final_padding = max_len + 1; // +1 space after label
        MAX_LABEL_WIDTH.store(final_padding, Ordering::SeqCst);
        final_padding
    }

    pub fn get_stored_padding_width() -> usize {
        MAX_LABEL_WIDTH.load(Ordering::SeqCst)
    }

    #[allow(dead_code)]
    fn truncate_str(s: &str, max_width: usize) -> String {
        let mut lines = s.lines();
        let first_line = lines.next().unwrap_or(s).trim_end_matches('\\').trim();
        let has_more_lines = lines.next().is_some();

        if first_line.len() < max_width && has_more_lines {
            format!("{}…", first_line)
        } else if first_line.len() <= max_width && !has_more_lines {
            first_line.to_string()
        } else {
            format!("{}…", &first_line[..max_width.saturating_sub(1)])
        }
    }

    #[allow(dead_code)]
    fn get_colored_label(
        label: &str,
        color_spec: Option<&ColorSpec>,
        color_enabled: bool,
    ) -> String {
        if !color_enabled {
            return label.to_string();
        }

        match color_spec {
            Some(ColorSpec::Black) => label.black().to_string(),
            Some(ColorSpec::Red) => label.red().to_string(),
            Some(ColorSpec::Green) => label.green().to_string(),
            Some(ColorSpec::Yellow) => label.yellow().to_string(),
            Some(ColorSpec::Blue) => label.blue().to_string(),
            Some(ColorSpec::Magenta) => label.magenta().to_string(),
            Some(ColorSpec::Cyan) => label.cyan().to_string(),
            Some(ColorSpec::White) => label.white().to_string(),
            Some(ColorSpec::BrightBlack) => label.bright_black().to_string(),
            Some(ColorSpec::BrightRed) => label.bright_red().to_string(),
            Some(ColorSpec::BrightGreen) => label.bright_green().to_string(),
            Some(ColorSpec::BrightYellow) => label.bright_yellow().to_string(),
            Some(ColorSpec::BrightBlue) => label.bright_blue().to_string(),
            Some(ColorSpec::BrightMagenta) => label.bright_magenta().to_string(),
            Some(ColorSpec::BrightCyan) => label.bright_cyan().to_string(),
            Some(ColorSpec::BrightWhite) => label.bright_white().to_string(),
            None => label.green().to_string(),
        }
    }

    #[allow(dead_code)]
    fn get_colored_output(output: &str, color_index: usize) -> String {
        let colors = ["blue", "green", "yellow", "red", "magenta", "cyan"];
        match colors[color_index] {
            "blue" => output.blue().to_string(),
            "green" => output.green().to_string(),
            "yellow" => output.yellow().to_string(),
            "red" => output.red().to_string(),
            "magenta" => output.magenta().to_string(),
            "cyan" => output.cyan().to_string(),
            _ => output.green().to_string(),
        }
    }
}

impl Default for PrintCommandPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl BodoPlugin for PrintCommandPlugin {
    fn on_command_ready(&self, command: &str, task_name: &str) {
        if !command.is_empty() {
            let prefix = if task_name.starts_with(".:") {
                format!("[{}]", task_name)
            } else {
                format!("> {}: ", task_name)
            };
            println!("{}{}", prefix, command);
        }
    }

    fn on_error(&self, task_name: &str, error: &dyn std::error::Error) {
        eprintln!("[{}] Error: {}", task_name.red(), error);
    }

    fn on_before_task_run(&self, _task_name: &str) {
        // Reset the header printed flag and command counter at the start of each task
        HEADER_PRINTED.store(false, Ordering::SeqCst);
        COMMAND_COUNTER.store(0, Ordering::SeqCst);
    }
}
