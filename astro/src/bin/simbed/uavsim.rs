use std::os::unix::net::UnixStream;
use std::cell::RefCell;
use std::time::Instant;
use std::rc::Rc;

use serde::Serialize;

use astro::comm;
use astro::kinetics::{self, PosVec, Velocity, KntcMsg, distance};
use astro::gps::{self, GpsMsg};
use astro::control::msg::{Nid, root_nid, Msg};
use astro::transceiver::Transceiver;

use super::uavconf::UavConf;

pub struct MsgPack {
    id: u32,
    p: PosVec,
    msg_out_distance: f32,
    data_vec: Vec<String>,
}

impl MsgPack {
    pub fn get_source_id(&self) -> u32 {
        self.id
    }

    pub fn get_source_p(&self) -> &PosVec {
        &self.p
    }
}

// used to record uav status in a file
#[derive(Clone, Serialize, Debug)]
pub struct UavInfo {
    pub nid: Nid,
    pub p: PosVec,
    pub v: Velocity,
}

// provides simulation support for a running UAV.
pub struct UavSim {
    conf: Rc<UavConf>,
    nid: Nid,
    p: PosVec,
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
            nid: root_nid(conf.id),
            p: conf.init_p,
            p_calc_t: now,
            p_send_t: now - conf.p_send_intrvl,
            v: Velocity::zero(),  // initialised with a dummy value
            tc: RefCell::new(Transceiver::new(stream)),
        }
    }

    pub fn get_id(&self) -> u32 {
        self.conf.id
    }

    pub fn get_info(&self) -> UavInfo {
        UavInfo {
            nid: self.nid.clone(),
            p: self.p,
            v: self.v,
        }
    }

    pub fn update_v(&mut self) -> bool {
        let mut updated: bool = false;
        if let Some(m) = self.tc.borrow_mut().retrieve::<KntcMsg>(kinetics::CHANNEL).last() {
            self.v = m.v;
            self.v.limit_norm_to(self.conf.max_v);
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

    pub fn collect_comm_msgs_and_update_nid(&mut self) -> MsgPack {  // collect messages from this UAV
        let data_vec = self.tc.borrow_mut().retrieve_raw(comm::CHANNEL);
        if let Some(d) = data_vec.last() {
            let msg: Msg = serde_json::from_str(d).unwrap();
            self.nid = msg.sender.nid;
        }
        MsgPack {
            id: self.conf.id,
            p: self.p,
            msg_out_distance: self.conf.msg_out_distance,
            data_vec,
        }
    }

    // receive messages from other UAVs, filtering by distance
    pub fn dispose_comm_msg_packs(&self, msg_packs: &Vec<MsgPack>) {
        for pack in msg_packs {
            if pack.id == self.conf.id {
                continue;  // filtering out messages sent by itself
            }
            if pack.msg_out_distance < distance(&pack.p, &self.p) {
                continue;  // filtering out messages sent by far-awary UAVs
            }
            for data in &pack.data_vec {
                let msg: Msg = serde_json::from_str(data).unwrap();
                if !self.should_receive_msg(&msg) {
                    continue;
                }
                self.tc.borrow_mut().send_raw(comm::CHANNEL, data);
            }
        }
    }

    pub fn dispose_comm_msgs(&self, msgs: &Vec<Msg>) {
        for msg in msgs {
            if !self.should_receive_msg(msg) {
                continue;
            }
            self.tc.borrow_mut().send(comm::CHANNEL, msg);
        }
    }

    pub fn should_receive_msg(&self, msg: &Msg) -> bool {
        msg.to_ids.is_empty() || msg.to_ids.contains(&self.conf.id)
    }

    pub fn overlap_with_uav_at(&self, other_p: &PosVec) -> bool {  // assuming same radius
        distance(&self.p, other_p) <= 2.0 * self.conf.radius
    }
}
