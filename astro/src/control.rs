use std::rc::Rc;
use std::collections::{HashMap, HashSet};

use super::astroconf::AstroConf;
use super::kinetics::{Position, Velocity};
use super::comm::CommMsg;

mod conn;
mod uav;

use uav::Uav;

// group id: (founder id, tag)
// group level: gid(top) -> gid -> gid -> gid -> ... -> gid(this)
pub struct Group {
    group_level: Vec<(u32, u32)>,
    connections: HashMap<u32, HashSet<u32>>,  // connection graph of the group
    uavs_in_reach: Vec<Uav>,
}

pub struct Control {
    conf: Rc<AstroConf>,
    next_group_tag: u32,
    swarm_size: u32,
}

impl Control {
    pub fn new(conf: &Rc<AstroConf>) -> Control {
        let group_tag = 0;
        Control {
            conf: conf.clone(),
            next_group_tag: group_tag + 1,
            swarm_size: 1,
        }
    }

    pub fn process(&mut self, p: &Position, v: &Velocity, msgs_in: &Vec<CommMsg>)
    -> (Velocity, Vec<CommMsg>) {
        let next_v = Velocity {vx: 0.0, vy: 0.0, vz: 0.0};
        let mut msgs_out: Vec<CommMsg> = vec![];
        (next_v, msgs_out)
    }
}