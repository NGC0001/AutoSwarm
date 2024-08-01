use std::rc::Rc;

use super::astroconf::AstroConf;
use super::kinetics::{PosVec, Velocity};

mod collivoid;
mod contacts;
mod msg;
mod nm;

pub use msg::Msg;

use collivoid::ColliVoid;
use contacts::Contacts;
use msg::NodeDesc;
use nm::NodeManager;

pub struct Control {
    conf: Rc<AstroConf>,
    contacts: Contacts,
    nm: NodeManager,
    collivoid: ColliVoid,
}

impl Control {
    pub fn new(conf: &Rc<AstroConf>, p: &PosVec, v: &Velocity) -> Control {
        Control {
            conf: conf.clone(),
            contacts: Contacts::new(p, conf.msg_range),
            nm: NodeManager::new_root_node(conf, p, v),
            collivoid: ColliVoid::new(conf),
        }
    }

    pub fn update(&mut self, p: &PosVec, v: &Velocity, msgs_in: &Vec<Msg>)
    -> (Velocity, Vec<Msg>) {
        let (neighbours, _, rm, mut msgs) = self.contacts.update(p, msgs_in);
        let neighbours_desc: Vec<&NodeDesc> = neighbours.iter().map(|ct| &ct.desc).collect();
        msgs.retain(|m| m.to_ids.contains(&self.conf.id));
        let (next_v, msgs_out) = self.nm.update_node(p, v, &rm, &msgs, &neighbours);
        // TODO: broadcasting
        let safe_v = self.collivoid.get_safe_v(&next_v, p, &neighbours_desc);
        (safe_v, msgs_out)
    }
}