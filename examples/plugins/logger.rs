use bodo::plugin::BodoPlugin;
use bodo::config::TaskConfig;

pub struct LoggerPlugin;

impl BodoPlugin for LoggerPlugin {
    fn on_before_task_run(&mut self, task_name: &str) {
        println!("[Logger] Starting task: {}", task_name);
    }

    fn on_after_task_run(&mut self, task_name: &str, status_code: i32) {
        println!("[Logger] Task {} finished with status {}", task_name, status_code);
    }

    fn on_error(&mut self, task_name: &str, err: &dyn std::error::Error) {
        eprintln!("[Logger] Task {} failed: {}", task_name, err);
    }

    fn on_resolve_command(&mut self, task: &mut TaskConfig) {
        if let Some(env) = &mut task.env {
            env.insert("DEBUG".to_string(), "1".to_string());
            println!("[Logger] Added DEBUG=1 to environment");
        }
    }

    fn on_command_ready(&mut self, command: &str, task_name: &str) {
        println!("[Logger] Executing command '{}' for task '{}'", command, task_name);
    }

    fn on_bodo_exit(&mut self, exit_code: i32) {
        println!("[Logger] Bodo exiting with code {}", exit_code);
    }
} 