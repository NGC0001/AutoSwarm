use super::msg::{NodeDesc, NodeDetails, Task, MsgBody, Msg};

pub struct TaskManager {
}

impl TaskManager {
    pub fn new() -> TaskManager {
        TaskManager {
        }
    }
}

pub fn calculate_task(task: &Task) {
    task.calc_length();
}