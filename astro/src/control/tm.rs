use std::collections::{HashMap, HashSet, VecDeque};
use std::option::Option;
use std::time::{Duration, Instant};

use super::super::kinetics::PosVec;

use super::msg::Task;

// executor monitors whether the uav is on assigned target position.
pub struct TaskExecutor {
    pos_target: PosVec,
    on_pos_t: Option<Instant>,
    succ_duration: Duration,
}

impl TaskExecutor {
    pub fn new(pos_target: &PosVec, succ_duration: Duration) -> TaskExecutor {
        TaskExecutor {
            pos_target: *pos_target,
            on_pos_t: None,
            succ_duration,
        }
    }

    pub fn execute(&mut self, pos: &PosVec) -> Option<bool> {
        unimplemented!("");
        self.get_result()
    }

    pub fn get_result(&self) -> Option<bool> {
        // result is never failure
        match self.on_pos_t {
            Some(on_pos_start_t) => {
                if Instant::now() - on_pos_start_t >= self.succ_duration {
                    Some(true)  // result is success
                } else {
                    None  // no result, still in progress
                }
            },
            None => None,  // no result, still in progress
        }
    }
}

// divider is responsible for dividing task into subtasks.
// subtasks include: a target position for this uav, tasks for children of this uav.
pub struct TaskDivider {
    task: Task,
    own_subtask: Option<TaskExecutor>,
    child_subtask: HashMap<u32, Task>,
}

impl TaskDivider {
    pub fn new(task: Task) -> TaskDivider {
        TaskDivider {
            task,
            own_subtask: None,
            child_subtask: HashMap::new(),
        }
    }

    pub fn get_tid(&self) -> u32 {
        self.task.id
    }

    pub fn divide_task(&mut self) {
        unimplemented!("")
    }

    pub fn is_task_divided(&self) -> bool {
        self.own_subtask.is_some()
    }

    pub fn get_own_subtask(&self) -> Option<&TaskExecutor> {
        self.own_subtask.as_ref()
    }

    pub fn get_own_subtask_mut(&mut self) -> Option<&mut TaskExecutor> {
        self.own_subtask.as_mut()
    }

    pub fn get_child_subtask(&self, cid: u32) -> Option<&Task> {
        self.child_subtask.get(&cid)
    }
}

pub struct TaskManager {
    task_exec: Option<TaskDivider>,
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

    pub fn get_current_task(&self) -> Option<&TaskDivider> {
        self.task_exec.as_ref()
    }

    pub fn get_current_task_mut(&mut self) -> Option<&mut TaskDivider> {
        self.task_exec.as_mut()
    }

    pub fn set_current_task(&mut self, task: Task) {
        self.task_exec = Some(TaskDivider::new(task));
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