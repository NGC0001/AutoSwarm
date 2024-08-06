use std::collections::{HashSet, VecDeque};
use std::option::Option;

use super::msg::Task;

pub struct TaskExecutor {
    task: Task,
}

impl TaskExecutor {
    pub fn new(task: Task) -> TaskExecutor {
        TaskExecutor {
            task,
        }
    }

    pub fn get_tid(&self) -> u32 {
        self.task.id
    }

    pub fn divide_task(&mut self) {
        unimplemented!("")
    }

    pub fn is_task_divided(&self) -> bool {
        unimplemented!("")
    }

    pub fn get_divided_subtask(&self, cid: u32) -> Option<&Task> {
        unimplemented!("")
    }

    pub fn execute(&mut self) {
        unimplemented!("")
    }

    pub fn is_task_success(&self) -> bool {
        unimplemented!("")
    }

    pub fn is_task_failure(&self) -> bool {
        unimplemented!("")
    }
}

pub struct TaskManager {
    task_exec: Option<TaskExecutor>,
    queued_tasks: VecDeque<Task>,
    old_tasks: HashSet<u32>,
}

impl TaskManager {
    pub fn new() -> TaskManager {
        TaskManager {
            task_exec: None,
            queued_tasks: VecDeque::<Task>::new(),
            old_tasks: HashSet::<u32>::new(),
        }
    }

    pub fn get_current_task(&self) -> Option<&TaskExecutor> {
        self.task_exec.as_ref()
    }

    pub fn get_current_task_mut(&mut self) -> Option<&mut TaskExecutor> {
        self.task_exec.as_mut()
    }

    pub fn set_current_task(&mut self, task: Task) {
        self.task_exec = Some(TaskExecutor::new(task));
    }

    pub fn clear_current_task(&mut self) {
        match &self.task_exec {
            Some(te) => {
                self.old_tasks.insert(te.get_tid());
                self.task_exec = None;
            },
            None => (),
        }
    }

    pub fn pop_queued_task(&mut self) -> Option<Task> {
        self.queued_tasks.pop_front()
    }

    pub fn is_task_new(&self, task: &Task) -> bool {
        !self.task_exec.as_ref().is_some_and(|te| te.get_tid() == task.id)
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