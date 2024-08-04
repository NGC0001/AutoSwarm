use super::msg::{NodeDesc, NodeDetails, Task, MsgBody, Msg};

pub fn calculate_task(task: &Task) {
    task.calc_length();
}