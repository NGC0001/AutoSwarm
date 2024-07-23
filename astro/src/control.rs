use std::rc::Rc;

use super::astroconf::AstroConf;
use super::kinetics::{Position, Velocity};
use super::comm::CommMsg;
use super::groupstate::{GrpId, GrpLevel, Sid, Member, GrpDesc, GrpState};

mod conn;
mod group;

use conn::Connection;
use group::Group;

pub struct Control {
    conf: Rc<AstroConf>,
    conn: Connection,
    next_tag: u32,
    group: Group,
}

impl Control {
    pub fn new(conf: &Rc<AstroConf>, p: &Position) -> Control {
        let init_tag = 0;
        Control {
            conf: conf.clone(),
            conn: Connection::new(p, conf.msg_range),
            next_tag: init_tag + 1,
            group: Group::new_soliton(conf.id, p, init_tag),
        }
    }

    pub fn process(&mut self, p: &Position, v: &Velocity, msgs_in: &Vec<CommMsg>)
    -> (Velocity, Vec<CommMsg>) {
        let (add, rm) = self.conn.update(p, msgs_in);
        let msgs_effective = self.conn.filter_messages(msgs_in);
        self.group.modify_conn_for(self.conf.id, &add, &rm);

        let next_v = Velocity {vx: 0.0, vy: 0.0, vz: 0.0};
        let mut msgs_out: Vec<CommMsg> = vec![];
        (next_v, msgs_out)
    }
}