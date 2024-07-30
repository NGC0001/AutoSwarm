use std::cmp::Ordering;
use std::option::Option;
use std::rc::Rc;
use std::time::{Duration, Instant};

use rand::Rng;

use super::super::astroconf::AstroConf;
use super::super::kinetics::{PosVec, Velocity};
use super::msg::{Nid, id_of, root_id_of, is_root_node, parent_id_of, valid_descendant_of};
use super::msg::{NodeDesc, MsgBody, Msg};

pub const DEFAULT_NODE_LOST_DURATION: Duration = Duration::from_secs(5);

struct Node {
    desc: NodeDesc,
    last_heard: Instant,
}

pub struct NodeManager {
    conf: Rc<AstroConf>,
    nid: Nid,
    parent: Option<Node>,  // need backup ids (indirect upper nodes / sibling nodes)
    children: Vec<Node>,
    node_lost_duration: Duration,
    p: PosVec,
    v: Velocity,
}

impl NodeManager {
    pub fn new_root_node(conf: &Rc<AstroConf>, p: &PosVec, v: &Velocity) -> NodeManager {
        NodeManager {
            conf: conf.clone(),
            nid: vec![conf.id],
            parent: None,
            children: vec![],
            node_lost_duration: DEFAULT_NODE_LOST_DURATION,
            p: *p,
            v: *v,
        }
    }

    pub fn is_root_node(&self) -> bool {
        is_root_node(&self.nid)
    }

    // TODO: this is a bad algorithm, and too long a function.
    pub fn join_other_tree(&mut self, candidates: &mut Vec<&NodeDesc>) -> Option<Msg> {
        // TODO: the number of children also needs considered
        candidates.sort_unstable_by(|desc1, desc2| {
            let swm_cmp = desc1.swm.cmp(&desc2.swm);
            match swm_cmp {
                Ordering::Equal => root_id_of(&desc2.nid).cmp(&root_id_of(&desc1.nid)),
                _ => swm_cmp,
            }
        });
        match candidates.last() {
            None => None,
            Some(desc) => {
                let (swarm_size, _) = self.get_swarm_size();
                let mut rng = rand::thread_rng();
                if desc.swm < swarm_size {
                    None
                } else if desc.swm == swarm_size && self.get_root_id() <= root_id_of(&desc.nid) {
                    None
                } else if !self.is_root_node() && rng.gen_range(0..10) < 5 {  // TODO: try to avoid random number
                    None
                } else {
                    // TODO: wait for swarm_size to update
                    let mut msg = Msg::new(self.generate_node_desc());
                    msg.to_ids.push(id_of(&desc.nid));
                    msg.body = MsgBody::JOIN;
                    // TODO: update code here and validate the parent
                    self.parent = Some(Node {
                        desc: (*desc).clone(),
                        last_heard: Instant::now(),
                    });
                    let id = id_of(&self.nid);
                    println!("{} -> {}", id_of(&desc.nid), id);
                    self.nid = desc.nid.clone();
                    self.nid.push(id);
                    Some(msg)
                }
            },
        }
    }

    pub fn get_root_id(&self) -> u32 {
        root_id_of(&self.nid)
    }

    pub fn calc_next_v(&self) -> Velocity {
        match &self.parent {
            None => Velocity::zero(),
            Some(pn) => {
                // TODO: this is a bad algorithm.
                let s = &pn.desc.p - &self.p;
                let dist: f32 = s.norm();
                if dist < self.conf.msg_range / 3.0 {
                    Velocity::zero()
                } else {
                    let factor: f32 = (self.conf.max_v / 2.0) / dist;
                    // TODO: take parent velocity into account
                    Velocity {
                        vx: s.x * factor,
                        vy: s.y * factor,
                        vz: s.z * factor,
                    }
                }
            },
        }
    }

    pub fn generate_desc_msg(&self) -> Msg {
        let mut to_ids: Vec<u32> = vec![];
        if let Some(nd) = &self.parent {
            to_ids.push(id_of(&nd.desc.nid));
        }
        for nd in &self.children {
            to_ids.push(id_of(&nd.desc.nid));
        }
        let mut msg = Msg::new(self.generate_node_desc());
        msg.to_ids.append(&mut to_ids);
        msg
    }

    pub fn get_swarm_size(&self) -> (u32, u32) {
        let mut subswarm_size: u32 = 1;
        for nd in &self.children {
            subswarm_size += nd.desc.subswm;
        }
        let swarm_size: u32 = match &self.parent {
            None => subswarm_size,
            Some(nd) => nd.desc.swm,
        };
        (swarm_size, subswarm_size)
    }

    pub fn generate_node_desc(&self) -> NodeDesc {
        let (swarm_size, subswarm_size) = self.get_swarm_size();
        NodeDesc {
            nid: self.nid.clone(),
            cids: self.children.iter().map(|nd| id_of(&nd.desc.nid)).collect(),
            p: self.p,
            v: self.v,
            subswm: subswarm_size,
            swm: swarm_size,
        }
    }

    pub fn update_node(&mut self, p: &PosVec, v: &Velocity, rm: &Vec<u32>, msgs: &Vec<&Msg>) {
        self.p = *p;
        self.v = *v;
        self.remove_no_connection_nodes(rm);
        let now = Instant::now();
        for msg in msgs {
            self.process_msg(now, msg);
        }
        self.remove_lost_nodes(now);
    }

    fn remove_no_connection_nodes(&mut self, rm: &Vec<u32>) {
        if self.parent.as_ref().is_some_and(|pnd| rm.contains(&id_of(&pnd.desc.nid))) {
            self.remove_parent();
        }
        self.children.retain(|ndesc| !rm.contains(&id_of(&ndesc.desc.nid)));
    }

    fn remove_parent(&mut self) {
        let id: u32 = id_of(&self.nid);
        self.parent = None;
        self.nid = vec![id];
    }

    fn process_msg(&mut self, now: Instant, msg: &Msg) {
        let desc_sdr = &msg.sender;
        self.update_desc(now, desc_sdr);
        match msg.body {
            MsgBody::KEEPALIVE => (),
            MsgBody::JOIN => self.add_child(now, desc_sdr),
            MsgBody::LEAVE => self.remove_child(id_of(&desc_sdr.nid)),
            MsgBody::TASK(..) => (),
        }
    }

    fn add_child(&mut self, now: Instant, desc: &NodeDesc) {
        let id_other = id_of(&desc.nid);
        if self.is_parent_or_child(id_other) {
            // the other node is already a recognised child
            return;
        }
        if !valid_descendant_of(id_other, &self.nid) {
            // the other node is not a valid child of this node
            return;
        }
        self.children.push(Node {
            desc: desc.clone(),
            last_heard: now,
        })
    }

    fn remove_child(&mut self, id_other: u32) {
        self.children.retain(|cnd| id_of(&cnd.desc.nid) != id_other);
    }

    fn is_parent_or_child(&self, id_other: u32) -> bool {
        self.is_parent(id_other) || self.is_child(id_other)
    }

    fn is_parent(&self, id_other: u32) -> bool {
        self.parent.as_ref().is_some_and(|pnd| id_of(&pnd.desc.nid) == id_other)
    }

    fn is_child(&self, id_other: u32) -> bool {
        self.children.iter().any(|cnd| id_of(&cnd.desc.nid) == id_other)
    }

    fn update_desc(&mut self, now: Instant, desc: &NodeDesc) {
        if !self.is_parent(id_of(&desc.nid)) {
            // the description is not from parent, check child descriptions
            self.update_child_desc(now, desc);
            return;
        }
        // the description is from recognised parent
        let pnd = self.parent.as_mut().unwrap();
        let id = id_of(&self.nid);
        if valid_descendant_of(id, &desc.nid) && desc.cids.contains(&id) {
            // valid parent,
            // and the parent recognise this node as its child.
            pnd.desc = desc.clone();
            pnd.last_heard = now;
            self.nid = desc.nid.clone();
            self.nid.push(id);
        }  // otherwise, just wait the parent to timeout and be removed
    }

    fn update_child_desc(&mut self, now: Instant, desc: &NodeDesc) {
        let id = id_of(&self.nid);
        for cnd in &mut self.children {
            let id_other = id_of(&desc.nid);
            if id_of(&cnd.desc.nid) == id_other {
                // the description is from a recognised child
                if valid_descendant_of(id_other, &self.nid)
                    && parent_id_of(&desc.nid).is_some_and(|v| v == id) {
                    // valid child,
                    // and the child recognised this node as its parent.
                    cnd.desc = desc.clone();
                    cnd.last_heard = now;
                }  // otherwise, just wait the child to timeout and be removed
                return;
            }
        }
    }

    fn remove_lost_nodes(&mut self, now: Instant) {
        if let Some(nd) = &self.parent {
            if now - nd.last_heard > self.node_lost_duration {
                self.remove_parent();
            }
        }
        self.children.retain(|nd| now - nd.last_heard <= self.node_lost_duration);
    }
}