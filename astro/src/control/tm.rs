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

    pub fn divide_task(&mut self, children_info: &Vec<ChildInfo>, comm_range: f32) {
        let (pos_own, line_groups) = self.divide_pos_own_and_line_groups(&children_info);
        if let Some(comm_pos) = &self.task.comm_point {
            assert!(distance(&pos_own, comm_pos) < comm_range);  // top node should not lose contact with its parent
        }
        self.own_subtask = Some(TaskExecutor::new(&pos_own, self.task.duration));
        for (cinfo, line_grp) in children_info.iter().zip(line_groups.into_iter()) {
            self.child_subtask.insert(cinfo.id, Task {
                id: self.task.id,
                lines: line_grp,
                duration: self.task.duration,
                comm_point: Some(pos_own.clone()),
            });
        }
    }

    fn divide_pos_own_and_line_groups(&self, children_info: &Vec<ChildInfo>) -> (PosVec, Vec<Vec<Line>>) {
        let subswm_size = children_info.iter().map(|ci| ci.subswm_size).sum::<u32>() + 1;
        let distrib_vec = self.distribute_uav_to_lines(subswm_size);
        let mut grp_sizes: Vec<u32> = vec![1];
        for cinfo in children_info {
            grp_sizes.push(cinfo.subswm_size);
        }
        let mut line_groups = Self::divide_line_groups(&self.task.lines, &distrib_vec, &grp_sizes);
        let mut line_grp_own = line_groups.remove(0);
        assert!(line_grp_own.len() == 1);
        let line_own = line_grp_own.remove(0);
        assert!(!line_own.start || !line_own.end);
        let pos_own: PosVec = if line_own.start { line_own.points.first().unwrap().clone() }
            else if line_own.end { line_own.points.last().unwrap().clone() }
            else { Self::divide_line(line_own, 0.5).0.points.last().unwrap().clone() };
        (pos_own, line_groups)
    }

    fn distribute_uav_to_lines(&self, subswm_size: u32) -> Vec<u32> {
        let lines = &self.task.lines;
        let mut distrib_vec: Vec<u32> = lines.iter().map(|l| l.num_least_uavs()).collect();
        let least_uavs = distrib_vec.iter().sum();
        assert!(subswm_size >= least_uavs);
        let len_vec: Vec<f32> = lines.iter().map(|l| l.calc_length()).collect();
        let end_points_vec: Vec<u32> = lines.iter().map(|l| l.num_end_points()).collect();
        for _ in 0..(subswm_size - least_uavs) {
            let ((distrib_max_load, _), _) = distrib_vec.iter_mut().zip(
                len_vec.iter()).zip(end_points_vec.iter()).max_by(
                    |((distrib1, len1), ep1), ((distrib2, len2), ep2)| {
                        let effective_uavs1 = (**distrib1 as f32) - (**ep1 as f32) / 2.0;
                        let effective_uavs2 = (**distrib2 as f32) - (**ep2 as f32) / 2.0;
                        (**len1 / effective_uavs1).partial_cmp(&(**len2 / effective_uavs2)).unwrap()
                    }).unwrap();
            *distrib_max_load += 1;
        }
        distrib_vec
    }

    fn divide_line_groups(lines: &Vec<Line>, distrib_vec: &Vec<u32>, grp_sizes: &Vec<u32>) -> Vec<Vec<Line>> {
        let mut line_groups: Vec<Vec<Line>> = vec![];
        let mut line_grp: Vec<Line> = vec![];
        let mut line_task: Line;
        let (mut line_idx, mut line, mut distrib) = (0, None, 0);
        let (mut grp_idx, mut uavs) = (0, 0);
        while line_idx < lines.len() && grp_idx < grp_sizes.len() {
            if distrib == 0 {  // assert!(line.is_none());
                (line, distrib) = (Some(lines[line_idx].clone()), distrib_vec[line_idx]);
            }
            if uavs == 0 {
                uavs = grp_sizes[grp_idx];
            }
            (line_task, line, distrib, uavs) = Self::split_line_for_uav_group(line.unwrap(), distrib, uavs);
            line_grp.push(line_task);
            if distrib == 0 {
                line_idx += 1;
            }
            if uavs == 0 {
                grp_idx += 1;
                line_groups.push(line_grp);
                line_grp = vec![];
            }
        }
        assert!(line_idx == lines.len() && grp_idx == grp_sizes.len() && distrib == 0 && uavs == 0);
        line_groups
    }

    // out: task_line_split_off, left_line_part, left_distrib, left_uavs
    fn split_line_for_uav_group(line: Line, distrib: u32, uavs: u32) -> (Line, Option<Line>, u32, u32) {
        if distrib <= uavs {
            (line, None, 0, uavs - distrib)
        } else {
            let left_distrib = distrib - uavs;
            let weight_split: f32 = if line.start { (uavs as f32) - 0.5 } else { uavs as f32 };
            let weight_left: f32 = if line.end { (left_distrib as f32) - 0.5 } else { left_distrib as f32 };
            let ratio = weight_split / (weight_split + weight_left);
            let (line_split, line_left) = Self::divide_line(line, ratio);
            (line_split, Some(line_left), left_distrib, 0)
        }
    }

    fn divide_line(mut line: Line, ratio: f32) -> (Line, Line) {  // divide a line into tow by ratio
        assert!(0.0 < ratio && ratio < 1.0);
        let len1 = line.calc_length() * ratio;
        let mut line2 = Line {
            points: vec![],
            start: false,  // breakpoint does not require uav
            end: line.end,
        };
        line.end = false;  // breakpoint does not require uav
        let mut cur_len: f32 = 0.0;
        let mut prev_len: f32 = 0.0;
        let mut idx: usize = 0;
        for i in 1..line.points.len() {  // find the two points immediately adjacent to the breakpoint
            prev_len = cur_len;
            cur_len += distance(&line.points[i], &line.points[i - 1]);
            if cur_len >= len1 {
                idx = i;
                break;
            }
        }
        let weight1 = cur_len - len1;
        let weight2 = len1 - prev_len;
        let breakpoint = (line.points[idx - 1] * weight1 + line.points[idx] * weight2) / (weight1 + weight2);
        line2.points.push(breakpoint.clone());
        line2.points.extend_from_slice(&line.points[idx..]);
        line.points.drain(idx..);
        line.points.push(breakpoint);
        (line, line2)
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