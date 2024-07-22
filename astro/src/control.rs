use std::rc::Rc;

use super::astroconf::AstroConf;
use super::kinetics::{Position, Velocity};
use super::comm::CommMsg;

mod conn;
mod group;

use conn::Connection;
use group::Group;

pub struct Control {
    conf: Rc<AstroConf>,
    conn: Connection,
    next_group_tag: u32,
    group: Group,
    swarm_size: u32,
}

impl Control {
    pub fn new(conf: &Rc<AstroConf>, p: &Position) -> Control {
        let group_tag = 0;
        Control {
            conf: conf.clone(),
            conn: Connection::new(p, conf.msg_range),
            next_group_tag: group_tag + 1,
            group: Group::new_soliton(conf.id, p, group_tag),
            swarm_size: 1,
        }
    }

    pub fn process(&mut self, p: &Position, v: &Velocity, msgs_in: &Vec<CommMsg>)
    -> (Velocity, Vec<CommMsg>) {
        let (add, rm) = self.conn.update(p, msgs_in);
        let next_v = Velocity {vx: 0.0, vy: 0.0, vz: 0.0};
        let mut msgs_out: Vec<CommMsg> = vec![];
        (next_v, msgs_out)
    }
}