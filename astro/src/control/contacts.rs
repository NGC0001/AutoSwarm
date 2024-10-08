use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::super::kinetics::{PosVec, distance};
use super::msg::{Msg, Nid, NodeDesc};

pub const DEFAULT_IN_RANGE_THRESHOLD: f32 = 0.9;
pub const DEFAULT_LOST_DURATION: Duration = Duration::from_secs(3);

pub struct Contact {
    pub desc: NodeDesc,
    pub last_heard: Instant,
}

impl Contact {
    pub fn predict_p(&self, now: Instant) -> PosVec {
        self.desc.p + self.desc.v * (now - self.last_heard)
    }
}

impl Contact {
    pub fn from_msg(msg_time: Instant, msg: &Msg) -> Contact {
        Contact {
            desc: msg.sender.clone(),
            last_heard: msg_time,
        }
    }

    pub fn update_from_msg(&mut self, msg_time: Instant, msg: &Msg) {
        self.desc = msg.sender.clone();
        self.last_heard = msg_time;
    }
}

// manage the contact with neighbour uavs.
pub struct Contacts {
    p_self: PosVec,
    contacts_in_range: HashMap<u32, Contact>,
    contact_range: f32,
    in_range_threshold: f32,
    lost_duration: Duration,
}

impl Contacts {
    pub fn new(p: &PosVec, contact_range: f32) -> Contacts {
        Contacts {
            p_self: *p,
            contacts_in_range: HashMap::new(),
            contact_range,
            in_range_threshold: DEFAULT_IN_RANGE_THRESHOLD,
            lost_duration: DEFAULT_LOST_DURATION,
        }
    }

    // `msgs_in`: all messages received by the communication module, ordered by arrival time
    //
    // with these input messages, pick out those nodes that go out of contact,
    // and those nodes that go into contact.
    // a node nearer than `in_range_threshold * contact_range` is considered into contact.
    //
    // `neighbours`: all nodes currently in contact
    // `add`: nodes newly into contact
    // `rm`: nodes newly out of contact
    // `msgs`: messages sent by nodes in contact, and by ground control station
    //
    // returned `add` and `rm` should not overlap.
    pub fn update<'a>(&mut self, p_self: &PosVec, msgs_in: &'a Vec<Msg>)
    -> (Vec<&Contact>, Vec<&'a Nid>, Vec<u32>, Vec<&'a Msg>) {
        self.p_self = *p_self;
        // m_map stores the newest message (with fresh position and nid) from a uav
        let mut m_map: HashMap<u32, &'a Msg> = HashMap::new();
        let mut msgs: Vec<&Msg> = vec![];
        for msg in msgs_in {
            if msg.sender.is_gcs() {  // ground control station is not an uav
                msgs.push(msg);
                continue;
            }
            m_map.entry(msg.sender.get_id())
                .and_modify(|m| { *m = msg; })
                .or_insert(msg);
        }
        let now = Instant::now();
        let (add, mut rm) = self.update_by_msg_positions(now, &m_map);
        rm.append(&mut self.filter_out_lost_contacts(now));  // should contain no duplicate ids
        msgs.append(&mut self.pick_messages_in_range(msgs_in));
        let neighbours = self.get_contacts();
        (neighbours, add, rm, msgs)
    }

    pub fn get_contacts(&self) -> Vec<&Contact> {
        self.contacts_in_range.iter().map(|(_, t)| t).collect()
    }

    // this algorithm does not ensure symmetry.
    // "a in contact with b" does not ensure "b in contact with a".
    fn update_by_msg_positions<'a>(&mut self, msg_time: Instant, m_map: &HashMap<u32, &'a Msg>)
    -> (Vec<&'a Nid>, Vec<u32>) {
        let mut add: Vec<&'a Nid> = vec![];
        let mut rm: Vec<u32> = vec![];
        for (id_other, msg) in m_map {
            let d = distance(&msg.sender.p, &self.p_self);
            match self.contacts_in_range.get_mut(id_other) {
                Some(t) => {
                    if d > self.contact_range {
                        // existing contact goes out of range
                        self.contacts_in_range.remove(id_other);
                        rm.push(*id_other);
                    } else {
                        // existing contact stays in range
                        t.update_from_msg(msg_time, msg);
                    }
                },
                None => {
                    if d <= self.contact_range * self.in_range_threshold {
                        // new contact goes into range
                        self.contacts_in_range.insert(*id_other, Contact::from_msg(msg_time, msg));
                        add.push(&msg.sender.nid);
                    }
                    // else: new contact out of range, irrelative
                },
            }
        }
        (add, rm)
    }

    fn filter_out_lost_contacts(&mut self, now: Instant) -> Vec<u32> {
        let mut rm: Vec<u32> = vec![];
        for (id, t) in &self.contacts_in_range {
            if now - t.last_heard > self.lost_duration {
                rm.push(*id);
            }
        }
        for id in &rm {
            self.contacts_in_range.remove(id);
        }
        rm
    }

    fn pick_messages_in_range<'a>(&self, msgs: &'a Vec<Msg>) -> Vec<&'a Msg> {
        let mut msg_in_range: Vec<&Msg> = vec![];
        for msg in msgs {
            match self.contacts_in_range.get(&msg.sender.get_id()) {
                None => (),
                Some(_) => {
                    msg_in_range.push(msg);
                }
            }
        }
        msg_in_range
    }
}