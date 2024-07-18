use std::os::unix::net::UnixStream;
use std::cell::RefCell;
use std::time::Instant;
use std::rc::Rc;

use astro::comm;
use astro::kinetics::{self, Position, Velocity, KntcMsg};
use astro::gps::{self, GpsMsg};
use astro::transceiver::Transceiver;

use super::uavconf::UavConf;

pub struct MsgPack {
    id: u32,
    p: Position,
    msg_out_distance: f32,
    data_vec: Vec<String>,
}

impl MsgPack {
    pub fn get_source_id(&self) -> u32 {
        self.id
    }

    pub fn get_source_p(&self) -> &Position {
        &self.p
    }
}

// provides simulation support for a running UAV.
pub struct UavSim {
    conf: Rc<UavConf>,
    p: Position,
    p_calc_t: Instant,
    p_send_t: Instant,
    v: Velocity,
    tc: RefCell<Transceiver>,
}

impl UavSim {
    pub fn new(conf: &Rc<UavConf>, stream: UnixStream) -> UavSim {
        let now = Instant::now();
        UavSim {
            conf: conf.clone(),
            p: conf.init_p,
            p_calc_t: now,
            p_send_t: now - conf.p_send_intrvl,
            v: Velocity {vx: 0.0, vy: 0.0, vz: 0.0},
            tc: RefCell::new(Transceiver::new(stream)),
        }
    }

    pub fn get_id(&self) -> u32 {
        self.conf.id
    }

    pub fn update_v(&mut self) -> bool {
        let mut updated: bool = false;
        for m in self.tc.borrow_mut().retrieve::<KntcMsg>(kinetics::CHANNEL) {
            self.v = m.v;
            updated = true;
        }
        updated
    }

    pub fn update_p(&mut self) {  // integration of v into p
        let now = Instant::now();
        self.p += &self.v * (now - self.p_calc_t);
        self.p_calc_t = now;
        if now - self.p_send_t > self.conf.p_send_intrvl {
            self.send_gps_msg();
            self.p_send_t = now;
        }
    }

    pub fn send_gps_msg(&self) {  // send position to UAV
        let msg = GpsMsg {p: self.p};
        self.tc.borrow_mut().send(gps::CHANNEL, &msg);
    }

    pub fn collect_comm_msgs(&self) -> MsgPack {  // collect messages from this UAV
        MsgPack {
            id: self.conf.id,
            p: self.p,
            msg_out_distance: self.conf.msg_out_distance,
            data_vec: self.tc.borrow_mut().retrieve_raw(comm::CHANNEL),
        }
    }

    // receive messages from other UAVs, filtering by distance
    pub fn dispose_comm_msgs(&self, msg_packs: &Vec<MsgPack>) {
        for pack in msg_packs {
            if pack.id == self.conf.id {
                continue;  // filtering out messages sent by itself
            }
            if pack.msg_out_distance < Self::calc_distance(&pack.p, &self.p) {
                continue;  // filtering out messages sent by far-awary UAVs
            }
            for msg in &pack.data_vec {
                self.tc.borrow_mut().send_raw(comm::CHANNEL, msg);
            }
        }
    }

    pub fn overlap_with_uav_at(&self, other_p: &Position) -> bool {  // assuming same radius
        Self::calc_distance(&self.p, other_p) <= 2.0 * self.conf.radius
    }

    fn calc_distance(p1: &Position, p2: &Position) -> f32 {
        (
            f32::powi(p1.x - p2.x, 2) + 
            f32::powi(p1.y - p2.y, 2) + 
            f32::powi(p1.z - p2.z, 2)
        ).sqrt()
    }
}
