use crate::config::load_script_config;
use crate::plugin::BodoPlugin;
use colored::Colorize;
use dialoguer::console::Term;
use std::sync::atomic::{AtomicUsize, Ordering};

static MAX_LABEL_WIDTH: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone)]
pub struct PrintCommandPlugin;

impl PrintCommandPlugin {
    pub fn new() -> Self {
        Self
    }

    fn get_max_width() -> usize {
        let term = Term::stdout();
        ((term.size().1 as f64) * 0.6) as usize
    }

    fn get_padding_width(concurrent_items: &[crate::config::ConcurrentItem], group: &str) -> usize {
        let mut max_len = 0;
        for item in concurrent_items {
            let label = match item {
                crate::config::ConcurrentItem::Task { task, .. } => {
                    format!("[{}:{}]", group, task)
                }
                crate::config::ConcurrentItem::Command { .. } => {
                    format!("[{}:command]", group)
                }
            };
            max_len = max_len.max(label.len());
        }
        let padding = max_len + 6;
        MAX_LABEL_WIDTH.store(padding, Ordering::SeqCst);
        padding
    }

    pub fn get_stored_padding_width() -> usize {
        MAX_LABEL_WIDTH.load(Ordering::SeqCst)
    }

    fn truncate_str(s: &str, max_width: usize) -> String {
        let mut lines = s.lines();
        let first_line = lines.next().unwrap_or(s).trim_end_matches('\\').trim();

        // Check if there are more lines
        let has_more = lines.next().is_some();

        if first_line.len() < max_width && has_more {
            format!("{}…", first_line)
        } else if first_line.len() <= max_width && !has_more {
            first_line.to_string()
        } else {
            format!("{}…", &first_line[..max_width - 1])
        }
    }

    fn get_colored_label(label: String) -> (String, usize) {
        // Get a deterministic color based on the label content
        let colors = ["blue", "green", "yellow", "red", "magenta", "cyan"];
        let color_index = label
            .chars()
            .fold(0usize, |acc, c| (acc + c as usize) % colors.len());

        let colored_label = match colors[color_index] {
            "blue" => label.blue().bold().to_string(),
            "green" => label.green().bold().to_string(),
            "yellow" => label.yellow().bold().to_string(),
            "red" => label.red().bold().to_string(),
            "magenta" => label.magenta().bold().to_string(),
            "cyan" => label.cyan().bold().to_string(),
            _ => label.green().bold().to_string(),
        };
        (colored_label, color_index)
    }

    fn get_colored_output(output: String, color_index: usize) -> String {
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
        // Don't print if the task is marked as silent
        if let Some((group, task)) = task_name.split_once(':') {
            if let Ok(script_config) = load_script_config(group) {
                if let Some(tasks) = &script_config.tasks {
                    if let Some(task_config) = tasks.get(task) {
                        if task_config.silent.unwrap_or(false) {
                            return;
                        }
                    }
                }

                // Check if this task is part of a concurrent group
                if let Some(concurrent_items) = &script_config.default_task.concurrently {
                    let concurrent_count = concurrent_items.len();
                    let max_width = Self::get_max_width();

                    // Print the header only for the first task
                    if task_name == format!("{}:{}", group, concurrent_items[0]) {
                        println!(
                            "{}",
                            format!("Running {} concurrent tasks:", concurrent_count).bold()
                        );
                        let padding_width = Self::get_padding_width(concurrent_items, group);
                        for item in concurrent_items {
                            match item {
                                crate::config::ConcurrentItem::Task { task, .. } => {
                                    if let Some(tasks) = &script_config.tasks {
                                        if let Some(task_config) = tasks.get(task) {
                                            let (colored_label, _) = Self::get_colored_label(
                                                format!("[{}:{}]", group, task),
                                            );
                                            println!(
                                                "{:<width$}{}",
                                                colored_label,
                                                Self::truncate_str(
                                                    task_config.command.as_deref().unwrap_or(""),
                                                    max_width
                                                )
                                                .dimmed(),
                                                width = padding_width
                                            );
                                        }
                                    }
                                }
                                crate::config::ConcurrentItem::Command { command, .. } => {
                                    let (colored_label, _) =
                                        Self::get_colored_label(format!("[{}:command]", group));
                                    println!(
                                        "{:<width$}{}",
                                        colored_label,
                                        Self::truncate_str(command, max_width).dimmed(),
                                        width = padding_width
                                    );
                                }
                            }
                        }
                        println!();
                    }
                    return;
                }
            }
        }

        let max_width = Self::get_max_width();
        let (colored_label, color_index) = Self::get_colored_label(task_name.to_string());
        println!(
            "{} {}: {}",
            ">".bold(),
            colored_label,
            Self::get_colored_output(Self::truncate_str(command, max_width), color_index)
        );
    }
}
