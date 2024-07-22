use std::collections::HashMap;
use std::time::Instant;

use super::Position;
use super::CommMsg;
use super::super::kinetics::distance;

pub const DEFAULT_IN_RANGE_THRESHOLD: f32 = 0.8;
pub const DEFAULT_OUT_RANGE_THRESHOLD: f32 = 0.9;

struct ConnRecord {
    p: Position,
    last_heard: Instant,
}

pub struct Connection {
    p_self: Position,
    in_range: HashMap<u32, ConnRecord>,
    msg_range: f32,
    in_range_threshold: f32,
    out_range_threshold: f32,
}

impl Connection {
    pub fn new(p_self: &Position, msg_range: f32) -> Connection {
        Connection {
            p_self: *p_self,
            in_range: HashMap::new(),
            msg_range,
            in_range_threshold: DEFAULT_IN_RANGE_THRESHOLD,
            out_range_threshold: DEFAULT_OUT_RANGE_THRESHOLD,
        }
    }

    // messages should be ordered by arrival time
    pub fn update(&mut self, p_self: &Position, msgs_in: &Vec<CommMsg>) {
        let mut d_map: HashMap<u32, &Position> = HashMap::new();
        let now = Instant::now();
        for msg in msgs_in {
            d_map.entry(msg.from_id).or_insert(&msg.from_p);
        }
        for (id, p_other) in d_map {
            let d = distance(p_other, p_self);
            match self.in_range.get_mut(&id) {
                Some(record) => {
                    if d > self.msg_range * self.out_range_threshold {
                        self.in_range.remove(&id);
                    } else {
                        record.last_heard = now;
                    }
                },
                None => {
                    if d < self.msg_range * self.in_range_threshold {
                        self.in_range.insert(id, ConnRecord {
                            p: *p_other,
                            last_heard: now,
                        });
                    }
                },
            }
        }
    }
}