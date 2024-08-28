use std::rc::Rc;
use std::time::{Duration, Instant};

use super::astroconf::AstroConf;
use super::kinetics::{PosVec, Velocity};

pub mod msg;

mod collivoid;
mod contacts;
mod nm;
mod tm;

use msg::Msg;
use collivoid::ColliVoid;
use contacts::Contacts;
use nm::NodeManager;

pub const DEFAULT_BROADCASTING_DURATION: Duration = Duration::from_millis(50);

pub struct Control {
    conf: Rc<AstroConf>,
    contacts: Contacts,
    nm: NodeManager,
    collivoid: ColliVoid,
    broadcasting_duration: Duration,
    last_broadcasting_t: Instant,
}

impl Control {
    pub fn new(conf: &Rc<AstroConf>, p: &PosVec, v: &Velocity) -> Control {
        let broadcasting_duration = DEFAULT_BROADCASTING_DURATION;
        let last_broadcasting_t = Instant::now() - broadcasting_duration;
        Control {
            conf: conf.clone(),
            contacts: Contacts::new(p, conf.contact_range),
            nm: NodeManager::new_root_node(conf, p, v),
            collivoid: ColliVoid::new(conf),
            broadcasting_duration,
            last_broadcasting_t,
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
        if now - self.last_broadcasting_t >= self.broadcasting_duration {
            msgs_out.push(Msg::new_empty_msg(self.nm.generate_node_desc()));
            self.last_broadcasting_t = now;
        }

        // collision avoidance module calculating safe velocity
        let safe_v = self.collivoid.get_safe_v(&next_v, p, &neighbours, now);

        (safe_v, msgs_out)
    }
}