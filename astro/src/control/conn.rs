use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::Position;
use super::msg::{Nid, Msg};
use super::super::kinetics::distance;

pub const DEFAULT_IN_RANGE_THRESHOLD: f32 = 0.8;
pub const DEFAULT_OUT_OF_RANGE_THRESHOLD: f32 = 0.9;
pub const DEFAULT_LOST_DURATION: Duration = Duration::from_secs(3);

struct Target {
    p: Position,
    last_heard: Instant,
}

// manage the connections with neighbour uavs.
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
    pub fn update<'a>(&mut self, p_self: &Position, msgs_in: &'a Vec<Msg>)
    -> (Vec<&'a Nid>, Vec<u32>) {
        self.p_self = *p_self;
        // m_map stores the newest message (with fresh position and sid) from a uav
        let mut m_map: HashMap<u32, &'a Msg> = HashMap::new();
        for msg in msgs_in {
            let from_id: u32 = *msg.from_nid.last().unwrap();
            m_map.entry(from_id)
                .and_modify(|p| { *p = msg; })
                .or_insert(msg);
        }
        let now = Instant::now();
        let (add, mut rm) = self.update_by_msg_positions(now, &m_map);
        self.filter_out_lost_targets(now);
        rm.append(&mut self.filter_out_lost_targets(now));  // should contain no duplicate ids
        (add, rm)
    }

    // this algorithm does not ensure symmetry.
    // "a in connection with b" does not ensure "b in connection with a".
    fn update_by_msg_positions<'a>(&mut self, msg_time: Instant, m_map: &HashMap<u32, &'a Msg>)
    -> (Vec<&'a Nid>, Vec<u32>) {
        let mut add: Vec<&'a Nid> = vec![];
        let mut rm: Vec<u32> = vec![];
        for (id_other, msg) in m_map {
            let d = distance(&msg.from_p, &self.p_self);
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
                            p: msg.from_p,
                            last_heard: msg_time,
                        });
                        add.push(&msg.from_nid);
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

    pub fn filter_messages<'a>(&self, msgs: &'a Vec<Msg>) -> Vec<&'a Msg> {
        let mut msg_filtered: Vec<&Msg> = vec![];
        for msg in msgs {
            let from_id: u32 = *msg.from_nid.last().unwrap();
            match self.targets_in_range.get(&from_id) {
                None => (),
                Some(_) => {
                    msg_filtered.push(msg);
                }
            }
        }
        msg_filtered
    }
}