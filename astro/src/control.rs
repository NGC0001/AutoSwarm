use std::rc::Rc;
use std::time::{Duration, Instant};

use super::astroconf::AstroConf;
use super::kinetics::{PosVec, Velocity};

mod collivoid;
mod contacts;
mod msg;
mod nm;

pub use msg::{Msg, MsgBody};

use collivoid::ColliVoid;
use contacts::Contacts;
use msg::NodeDesc;
use nm::NodeManager;

pub const DEFAULT_MAX_MSG_DURATION: Duration = Duration::from_millis(100);

pub struct Control {
    conf: Rc<AstroConf>,
    contacts: Contacts,
    nm: NodeManager,
    collivoid: ColliVoid,
    max_msg_duration: Duration,
    last_msg_t: Instant,
}

impl Control {
    pub fn new(conf: &Rc<AstroConf>, p: &PosVec, v: &Velocity) -> Control {
        let max_msg_duration = DEFAULT_MAX_MSG_DURATION;
        let last_msg_t = Instant::now() - max_msg_duration;
        Control {
            conf: conf.clone(),
            contacts: Contacts::new(p, conf.msg_range),
            nm: NodeManager::new_root_node(conf, p, v),
            collivoid: ColliVoid::new(conf),
            max_msg_duration,
            last_msg_t,
        }
    }

    pub fn update(&mut self, p: &PosVec, v: &Velocity, msgs_in: &Vec<Msg>)
    -> (Velocity, Vec<Msg>) {
        // with messages received, check nodes that are in contact
        let (neighbours, _, rm, mut msgs) = self.contacts.update(p, msgs_in);

        // keep only those messages sent specifically to this node
        msgs.retain(|m| m.to_ids.contains(&self.conf.id));
        // node manager update, generating output messages and giving an appropriate velocity
        let (next_v, mut msgs_out) = self.nm.update_node(p, v, &rm, &msgs, &neighbours);

        let now = Instant::now();
        if now - self.last_msg_t >= self.max_msg_duration && msgs_out.is_empty() {
            msgs_out.push(Msg {
                sender: self.nm.generate_node_desc(),
                to_ids: vec![],
                body: MsgBody::Empty,
            });
        }
        if !msgs_out.is_empty() {
            self.last_msg_t = now;
        }

        let neighbours_desc: Vec<&NodeDesc> = neighbours.iter().map(|ct| &ct.desc).collect();
        // collision avoidance module calculating safe velocity
        let safe_v = self.collivoid.get_safe_v(&next_v, p, &neighbours_desc);

        (safe_v, msgs_out)
    }
}