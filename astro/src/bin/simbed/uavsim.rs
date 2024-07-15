use std::os::unix::net::UnixStream;
use std::cell::RefCell;
use std::time::{Duration, Instant};
use std::rc::Rc;

use astro::comm;
use astro::control::{self, ControlMsg, Velocity};
use astro::gps::{self, GpsMsg, Position};
use astro::transceiver::Transceiver;

pub struct MsgPack {
    id: u32,
    p: Position,
    msg_out_distance: f64,
    data_vec: Vec<String>,
}

pub const DEFAULT_MSG_OUT_DISTANCE: f64 = 1000.0;

pub struct UavConf {
    pub id: u32,
    pub msg_out_distance: f64,  // how far away this UAV can transmit its messages
    pub init_p: Position,
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
            p_send_t: now - Duration::from_secs(3600 * 24 * 365 * 10),
            v: Velocity {vx: 0.0, vy: 0.0, vz: 0.0},
            tc: RefCell::new(Transceiver::new(stream)),
        }
    }

    pub fn update_v(&mut self) -> bool {
        let mut updated: bool = false;
        for m in self.tc.borrow_mut().retrieve::<ControlMsg>(control::CHANNEL) {
            self.v = m.v;
            updated = true;
        }
        updated
    }

    pub fn calc_p(&mut self) {  // integration of v into p
        let now = Instant::now();
        let dt = now - self.p_calc_t;
        let dt_s: f64 = dt.as_secs_f64();
        self.p.x += self.v.vx * dt_s;
        self.p.y += self.v.vy * dt_s;
        self.p.z += self.v.vz * dt_s;
        self.p_calc_t = now;
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

    fn calc_distance(p1: &Position, p2: &Position) -> f64 {
        (
            f64::powi(p1.x - p2.x, 2) + 
            f64::powi(p1.y - p2.y, 2) + 
            f64::powi(p1.z - p2.z, 2)
        ).sqrt()
    }
}