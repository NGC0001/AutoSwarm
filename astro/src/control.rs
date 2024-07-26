use std::rc::Rc;

use super::astroconf::AstroConf;
use super::kinetics::{Position, Velocity};

mod conn;
mod msg;
mod nm;

pub use msg::Msg;

use conn::Connection;
use msg::root_id_of;
use nm::NodeManager;

pub struct Control {
    conf: Rc<AstroConf>,
    conn: Connection,
    nm: NodeManager,
}

impl Control {
    pub fn new(conf: &Rc<AstroConf>, p: &Position, v: &Velocity) -> Control {
        Control {
            conf: conf.clone(),
            conn: Connection::new(p, conf.msg_range),
            nm: NodeManager::new_root_node(conf, p, v),
        }
    }

    pub fn update(&mut self, p: &Position, v: &Velocity, msgs_in: &Vec<Msg>)
    -> (Velocity, Vec<Msg>) {
        let (_, rm) = self.conn.update(p, msgs_in);
        let msgs_in_range = self.conn.pick_messages_in_range(msgs_in);
        let msgs = self.pick_messages(&msgs_in_range);
        self.nm.update_node(p, v, &rm, &msgs);

        let mut msgs_out: Vec<Msg> = vec![];
        msgs_out.push(self.nm.generate_desc_msg());

        let root_self = self.nm.get_root_id();
        let mut nodes_on_other_trees = self.conn.get_targets(|desc| root_id_of(&desc.nid) != root_self);
        if let Some(msg) = self.nm.join_other_tree(&mut nodes_on_other_trees) {
            msgs_out.push(msg);
        }

        dbg!(self.nm.get_nid());

        let next_v = self.nm.calc_next_v();
        (next_v, msgs_out)
    }

    fn pick_messages<'a>(&self, msgs_all: &Vec<&'a Msg>) -> Vec<&'a Msg> {
        let mut msgs: Vec<&Msg> = vec![];
        for msg in msgs_all {
            if msg.to_ids.contains(&self.conf.id) {
                msgs.push(msg);
            }
        }
        msgs
    }
}