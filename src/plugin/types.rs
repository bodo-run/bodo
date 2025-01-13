use crate::config::TaskConfig;
use std::error::Error;

pub trait BodoPlugin {
    fn on_before_task_run(&mut self, _task_name: &str) {}
    fn on_after_task_run(&mut self, _task_name: &str, _status_code: i32) {}
    fn on_error(&mut self, _task_name: &str, _err: &dyn Error) {}
    fn on_resolve_command(&mut self, _task: &mut TaskConfig) {}
    fn on_command_ready(&mut self, _command: &str, _task_name: &str) {}
    fn on_bodo_exit(&mut self, _exit_code: i32) {}
}
