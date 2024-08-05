use std::collections::{HashSet, VecDeque};
use std::option::Option;

use super::msg::Task;

pub struct TaskManager {
    task: Option<Task>,
    queued_tasks: VecDeque<Task>,
    old_tasks: HashSet<u32>,
}

impl TaskManager {
    pub fn new() -> TaskManager {
        TaskManager {
            task: None,
            queued_tasks: VecDeque::<Task>::new(),
            old_tasks: HashSet::<u32>::new(),
        }
    }

    pub fn get_current_task(&self) -> Option<&Task> {
        self.task.as_ref()
    }

    pub fn clear_current_task(&mut self) {
        match &self.task {
            Some(t) => {
                self.old_tasks.insert(t.id);
                self.task = None;
            },
            None => (),
        }
    }

    pub fn pop_queued_task(&mut self) -> Option<Task> {
        self.queued_tasks.pop_front()
    }

    pub fn is_task_new(&self, task: &Task) -> bool {
        !self.task.as_ref().is_some_and(|t| t.id == task.id)
        && !self.old_tasks.contains(&task.id)
        && self.queued_tasks.iter().all(|t| t.id != task.id)
    }

    pub fn add_task_if_new(&mut self, task: &Task) -> bool {
        if self.is_task_new(task) {
            self.queued_tasks.push_back(task.clone());
            true
        } else {
            false
        }
    }
}

pub fn calculate_task(task: &Task) {
    task.calc_length();
}