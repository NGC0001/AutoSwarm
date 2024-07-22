use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::Position;
use super::CommMsg;
use super::super::kinetics::distance;

pub const DEFAULT_IN_RANGE_THRESHOLD: f32 = 0.8;
pub const DEFAULT_OUT_OF_RANGE_THRESHOLD: f32 = 0.9;
pub const DEFAULT_LOST_DURATION: Duration = Duration::from_secs(3);

struct Target {
    p: Position,
    last_heard: Instant,
}

pub struct Connection {
    p_self: Position,
    targets_in_range: HashMap<u32, Target>,
    msg_range: f32,
    in_range_threshold: f32,
    out_of_range_threshold: f32,
    lost_duration: Duration,
}

impl Connection {
    pub fn new(p: &Position, msg_range: f32) -> Connection {
        Connection {
            p_self: *p,
            targets_in_range: HashMap::new(),
            msg_range,
            in_range_threshold: DEFAULT_IN_RANGE_THRESHOLD,
            out_of_range_threshold: DEFAULT_OUT_OF_RANGE_THRESHOLD,
            lost_duration: DEFAULT_LOST_DURATION,
        }
    }

    // messages should be ordered by arrival time
    pub fn update(&mut self, p_self: &Position, msgs_in: &Vec<CommMsg>)
    -> (Vec<u32>, Vec<u32>) {
        self.p_self = *p_self;
        let mut p_map: HashMap<u32, &Position> = HashMap::new();
        for msg in msgs_in {
            p_map.entry(msg.from_id)
                .and_modify(|p| { *p = &msg.from_p; })
                .or_insert(&msg.from_p);
        }
        let now = Instant::now();
        let (add, mut rm) = self.update_by_msg_positions(now, &p_map);
        self.filter_out_lost_targets(now);
        rm.append(&mut self.filter_out_lost_targets(now));  // should contain no duplicate ids
        (add, rm)
    }

    fn update_by_msg_positions(&mut self, msg_time: Instant, p_map: &HashMap<u32, &Position>)
    -> (Vec<u32>, Vec<u32>) {
        let mut add: Vec<u32> = vec![];
        let mut rm: Vec<u32> = vec![];
        for (id_other, p_other) in p_map {
            let d = distance(p_other, &self.p_self);
            match self.targets_in_range.get_mut(id_other) {
                Some(t) => {
                    if d > self.msg_range * self.out_of_range_threshold {
                        self.targets_in_range.remove(id_other);
                        rm.push(*id_other);
                    } else {
                        t.last_heard = msg_time;
                    }
                },
                None => {
                    if d <= self.msg_range * self.in_range_threshold {
                        self.targets_in_range.insert(*id_other, Target {
                            p: **p_other,
                            last_heard: msg_time,
                        });
                        add.push(*id_other);
                    }
                },
            }
        }
        (add, rm)
    }

    fn filter_out_lost_targets(&mut self, now: Instant) -> Vec<u32> {
        let mut rm: Vec<u32> = vec![];
        for (id, t) in &self.targets_in_range {
            if now - t.last_heard > self.lost_duration {
                rm.push(*id);
            }
        }
        for id in &rm {
            self.targets_in_range.remove(id);
        }
        rm
    }
}