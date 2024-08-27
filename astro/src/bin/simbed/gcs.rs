use std::fs::read_to_string;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use astro::kinetics::PosVec;
use astro::control::msg::{NodeDesc, MsgBody, Msg, Line, Task};

#[derive(Deserialize, Serialize, Debug)]
struct TaskInfo {
    task: Task,
    to_ids: Vec<u32>,
    wait_duration: Duration,
}

impl TaskInfo {
    pub fn demo_simple_line() -> TaskInfo {
        TaskInfo {
            task: Task {
                id: 0,
                lines: vec![Line {
                    points: vec![
                        PosVec {x: 0.0, y: 10.0, z: 10.0},
                        PosVec {x: 0.0, y: 20.0, z: 10.0},
                    ],
                    start: true,
                    end: true,
                }],
                duration: Duration::from_secs(10),
                comm_point: None,
            },
            to_ids: vec![2, 3],
            wait_duration: Duration::from_secs(10),
        }
    }

    pub fn demo_love() -> TaskInfo {
        TaskInfo {
            task: Task {
                id: 1,
                lines: vec![  // bottom-left (0, 2, 2), letter size (0, 4, 8), stride (0, 5, 0).
                    Line {  // 'L', (0, 2, 10)
                        points: vec![
                            PosVec {x: 0.0, y: 2.0, z: 10.0},
                            PosVec {x: 0.0, y: 2.0, z: 2.0},
                            PosVec {x: 0.0, y: 6.0, z: 2.0},
                        ],
                        start: true,
                        end: true,
                    },
                    Line {  // 'O', (0, 7, 10)
                        points: Self::create_ellipse(9.0, 6.0, 2.0, 4.0, 20),
                        start: false,
                        end: false,
                    },
                    Line {  // 'V', (0, 12, 10)
                        points: vec![
                            PosVec {x: 0.0, y: 12.0, z: 10.0},
                            PosVec {x: 0.0, y: 14.0, z: 2.0},
                            PosVec {x: 0.0, y: 16.0, z: 10.0},
                        ],
                        start: true,
                        end: true,
                    },
                    Line {  // 'E':'C', (0, 17, 10)
                        points: vec![
                            PosVec {x: 0.0, y: 21.0, z: 10.0},
                            PosVec {x: 0.0, y: 17.0, z: 10.0},
                            PosVec {x: 0.0, y: 17.0, z: 2.0},
                            PosVec {x: 0.0, y: 21.0, z: 2.0},
                        ],
                        start: true,
                        end: true,
                    },
                    Line {  // 'E':'-', (0, 17, 10)
                        points: vec![
                            PosVec {x: 0.0, y: 18.0, z: 6.0},
                            PosVec {x: 0.0, y: 21.0, z: 6.0},
                        ],
                        start: false,
                        end: true,
                    },
                ],
                duration: Duration::from_secs(10),
                comm_point: None,
            },
            to_ids: vec![3],
            wait_duration: Duration::from_secs(10),
        }
    }

    fn create_ellipse(cy: f32, cz: f32, ry: f32, rz: f32, n: u32) -> Vec<PosVec> {
        let mut points: Vec<PosVec> = vec![];
        for i in 0..n {
            let theta = (i as f32) * 2.0 * std::f32::consts::PI / (n as f32);
            let sin = theta.sin();
            let cos = theta.cos();
            let r = (rz * ry) / ((rz * sin).powi(2) + (ry * cos).powi(2)).sqrt();
            points.push(PosVec {
                x: 0.0,
                y: cy + r * sin,
                z: cz + r * cos,
            });
        }
        points
    }
}

pub struct Gcs {
    tasks: Vec<TaskInfo>,
}

impl Gcs {
    pub fn new(task_book: &String) -> Gcs {
        let mut tasks: Vec<TaskInfo> = vec![];
        if task_book.is_empty() || task_book == "demo_simple_line" {
            tasks.push(TaskInfo::demo_simple_line());
        } else if task_book == "demo_love" {
            tasks.push(TaskInfo::demo_love());
        } else {
            for line in read_to_string(task_book).unwrap().lines() {
                let task_info: TaskInfo = serde_json::from_str(line).unwrap();
                tasks.push(task_info);
            }
        }
        Gcs {
            tasks,
        }
    }

    pub fn generate_gcs_msgs(&mut self, running_duration: Duration) -> Vec<Msg> {
        let mut msgs: Vec<Msg> = vec![];
        let mut dispatched: Vec<u32> = vec![];
        for ti in &self.tasks {
            if running_duration >= ti.wait_duration {
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