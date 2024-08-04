use std::fs::read_to_string;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use astro::control::msg::{NodeDesc, MsgBody, Msg, Task};

#[derive(Deserialize, Serialize, Debug)]
struct TaskInfo {
    task: Task,
    to_ids: Vec<u32>,
    wait_duration: Duration,
}

pub struct Gcs {
    start_t: Instant,
    tasks: Vec<TaskInfo>,
}

impl Gcs {
    pub fn new(task_book: &String) -> Gcs {
        let mut tasks: Vec<TaskInfo> = vec![];
        if !task_book.is_empty() {
            for line in read_to_string(task_book).unwrap().lines() {
                let task_info: TaskInfo = serde_json::from_str(line).unwrap();
                tasks.push(task_info);
            }
        }
        Gcs {
            start_t: Instant::now(),
            tasks,
        }
    }

    pub fn generate_gcs_msgs(&mut self, now: Instant) -> Vec<Msg> {
        let mut msgs: Vec<Msg> = vec![];
        let mut dispatched: Vec<u32> = vec![];
        for ti in &self.tasks {
            if now - self.start_t >= ti.wait_duration {
                msgs.push(Msg {
                    sender: NodeDesc::get_gcs_desc(),
                    to_ids: ti.to_ids.clone(),
                    body: MsgBody::Task(ti.task.clone()),
                });
                dispatched.push(ti.task.id);
            }
        }
        self.tasks.retain(|t| !dispatched.contains(&t.task.id));
        msgs
    }
}