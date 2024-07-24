use std::rc::Rc;

use super::astroconf::AstroConf;
use super::kinetics::{Position, Velocity};

mod conn;
mod msg;
mod nm;

pub use msg::{Nid, Msg};

use conn::Connection;
use nm::NodeManager;

pub struct Control {
    conf: Rc<AstroConf>,
    conn: Connection,
    nm: NodeManager,
}

impl Control {
    pub fn new(conf: &Rc<AstroConf>, p: &Position) -> Control {
        Control {
            conf: conf.clone(),
            conn: Connection::new(p, conf.msg_range),
            nm: NodeManager::new_root(conf.id, p),
        }
    }

    pub fn process(&mut self, p: &Position, v: &Velocity, msgs_in: &Vec<Msg>)
    -> (Velocity, Vec<Msg>) {
        let (add, rm) = self.conn.update(p, msgs_in);
        let msgs_in_range = self.conn.filter_messages(msgs_in);
        self.nm.update_node_conn(p, &rm);

        let next_v = Velocity {vx: 0.0, vy: 0.0, vz: 0.0};
        let mut msgs_out: Vec<Msg> = vec![];
        msgs_out.push(self.nm.generate_desc_msg());
        (next_v, msgs_out)
    }
}