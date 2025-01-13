use crate::config::TaskConfig;
use std::error::Error;

pub trait BodoPlugin: Clone {
    fn on_before_task_run(&self, _task_name: &str) {}
    fn on_after_task_run(&self, _task_name: &str, _status_code: i32) {}
    fn on_error(&self, _task_name: &str, _err: &dyn Error) {}
    fn on_resolve_command(&self, _task: &TaskConfig) {}
    fn on_command_ready(&self, _command: &str, _task_name: &str) {}
    fn on_bodo_exit(&self, _exit_code: i32) {}
}
