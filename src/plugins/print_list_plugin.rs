use std::any::Any;

use crate::plugin::Plugin;

// Task info represents (task_name, description, script_id)
type TaskInfo = (String, Option<String>, String);
type ScriptTasks = Vec<(String, Vec<TaskInfo>)>;

struct TaskLine {
    #[allow(dead_code)]
    script_name: String,
    is_heading: bool,
    left_col: String,
    desc: Option<String>,
}

pub struct PrintListPlugin;

impl Plugin for PrintListPlugin {
    fn name(&self) -> &'static str {
        "PrintListPlugin"
    }

    fn priority(&self) -> i32 {
        0
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    // Rest of existing implementation...
}
