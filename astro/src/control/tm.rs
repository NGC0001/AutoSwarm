use std::collections::{HashMap, HashSet, VecDeque};
use std::option::Option;
use std::time::{Duration, Instant};

use super::super::kinetics::{distance, PosVec};

use super::msg::{Line, Task};

pub const DEFAULT_POS_MAINTAIN_PRECISION: f32 = 0.5;

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

    pub fn advance(&mut self, pos: &PosVec, now: Instant) -> Option<bool> {
        if distance(pos, &self.pos_target) <= DEFAULT_POS_MAINTAIN_PRECISION {
            if self.on_pos_t.is_none() {
                self.on_pos_t = Some(now);
            }
        } else {
            self.on_pos_t = None;
        }
        self.get_result(now)
    }

    pub fn get_result(&self, now: Instant) -> Option<bool> {
        // result is never failure
        match self.on_pos_t {
            Some(on_pos_start_t) => {
                if now - on_pos_start_t >= self.succ_duration {
                    Some(true)  // result is success
                } else {
                    None  // no result, still in progress
                }
            },
            None => None,  // no result, still in progress
        }
    }
}

pub struct ChildInfo {
    pub id: u32,
    pub subswm_size: u32,
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

    pub fn divide_task(&mut self, children_info: &Vec<ChildInfo>) {
        let subswm_size = children_info.iter().map(|ci| ci.subswm_size).sum::<u32>() + 1;
        let mut distrib_vec: Vec<u32> = self.task.lines.iter().map(|l| l.num_least_uavs()).collect();
        let least_uavs = distrib_vec.iter().sum();
        assert!(subswm_size >= least_uavs);
        let len_vec: Vec<f32> = self.task.lines.iter().map(|l| l.calc_length()).collect();
        let end_points_vec: Vec<u32> = self.task.lines.iter().map(|l| l.num_end_points()).collect();
        for _ in 0..(subswm_size - least_uavs) {
            let (_, distrib) = distrib_vec.iter_mut().enumerate().max_by(
                |(idx1, distrib1), (idx2, distrib2)| {
                    let effective_uavs1 = (**distrib1 as f32) - (end_points_vec[*idx1] as f32) / 2.0;
                    let effective_uavs2 = (**distrib2 as f32) - (end_points_vec[*idx2] as f32) / 2.0;
                    (len_vec[*idx1] / effective_uavs1).partial_cmp(&(len_vec[*idx2] / effective_uavs2)).unwrap()
                }).unwrap();
            *distrib += 1;
        }
        let mut cur_idx: usize = 0;
        let mut cur_distrib: u32 = distrib_vec[cur_idx];
        let mut cur_line: Line = self.task.lines[cur_idx].clone();
        for (cidx, cinfo) in children_info.iter().enumerate() {
            let mut subtask_lines: Vec<Line> = vec![];
            let mut spare_uavs = cinfo.subswm_size;
            while 0 < spare_uavs {
                if cur_distrib <= spare_uavs {
                    spare_uavs -= cur_distrib;
                    subtask_lines.push(cur_line);
                    cur_idx += 1;
                    cur_distrib = distrib_vec[cur_idx];
                    cur_line = self.task.lines[cur_idx].clone();
                } else {
                    cur_distrib -= spare_uavs;
                    let ratio1: f32 = if cur_line.start { (spare_uavs as f32) - 0.5 } else { spare_uavs as f32 };
                    let ratio2: f32 = if cur_line.end { (cur_distrib as f32) - 0.5 } else { cur_distrib as f32 };
                    let (line_subtask, line_remain) = Self::divide_line(cur_line, ratio1, ratio2);
                    spare_uavs = 0;
                    subtask_lines.push(line_subtask);
                    cur_line = line_remain;
                }
            }
        }
        assert!(cur_idx == self.task.lines.len() - 1);
        assert!(cur_distrib == 1);
        assert!(!cur_line.start || !cur_line.end);
        if !cur_line.start && !cur_line.end {
            cur_line = Self::divide_line(cur_line, 0.5, 0.5).0;  // take the first half
            cur_line.end = true;  // so the center of the whole is at the end of the half
        }
        let pos_own: &PosVec = if cur_line.start { &cur_line.points.first().unwrap() } else { &cur_line.points.last().unwrap() };
    }

    fn divide_line(line: Line, ratio1: f32, ratio2: f32) -> (Line, Line) {}

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