use std::rc::Rc;

use super::astroconf::AstroConf;
use super::kinetics::{Position, Velocity};

mod conn;
mod msg;

pub use msg::{Nid, Msg};

use conn::Connection;

pub struct Control {
    conf: Rc<AstroConf>,
    conn: Connection,
    next_tag: u32,
}

impl Control {
    pub fn new(conf: &Rc<AstroConf>, p: &Position) -> Control {
        let init_tag = 0;
        Control {
            conf: conf.clone(),
            conn: Connection::new(p, conf.msg_range),
            next_tag: init_tag + 1,
        }
    }

    pub fn process(&mut self, p: &Position, v: &Velocity, msgs_in: &Vec<Msg>)
    -> (Velocity, Vec<Msg>) {
        let (add, rm) = self.conn.update(p, msgs_in);
        let msgs_effective = self.conn.filter_messages(msgs_in);

        let next_v = Velocity {vx: 0.0, vy: 0.0, vz: 0.0};
        let mut msgs_out: Vec<Msg> = vec![];
        (next_v, msgs_out)
    }
}