use crate::config::load_script_config;
use crate::plugin::BodoPlugin;
use colored::Colorize;
use dialoguer::console::Term;

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
                        println!("Running {} concurrent tasks:", concurrent_count);
                        for item in concurrent_items {
                            match item {
                                crate::config::ConcurrentItem::Task { task, .. } => {
                                    if let Some(tasks) = &script_config.tasks {
                                        if let Some(task_config) = tasks.get(task) {
                                            println!(
                                                "{:<9}: {}",
                                                format!("{}:{}", group, task).green(),
                                                Self::truncate_str(
                                                    task_config.command.as_deref().unwrap_or(""),
                                                    max_width
                                                )
                                                .dimmed()
                                            );
                                        }
                                    }
                                }
                                crate::config::ConcurrentItem::Command { command, .. } => {
                                    println!(
                                        "{:<9}: {}",
                                        format!("{}:command", group).green(),
                                        Self::truncate_str(command, max_width).dimmed()
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
        println!(
            "> {}: {}",
            task_name.green(),
            Self::truncate_str(command, max_width).dimmed()
        );
    }
}
