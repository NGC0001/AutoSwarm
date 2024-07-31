use std::cmp::Ordering;
use std::option::Option;
use std::rc::Rc;
use std::time::{Duration, Instant};

use rand::Rng;

use super::super::astroconf::AstroConf;
use super::super::kinetics::{PosVec, Velocity};
use super::msg::{Nid, id_of, root_id_of, parent_id_of, valid_descendant_of};
use super::msg::{NodeDesc, NodeDetails, MsgBody, Msg};

pub const DEFAULT_NODE_LOST_DURATION: Duration = Duration::from_secs(5);
pub const DEFAULT_CONNECTION_MSG_DURATION: Duration = Duration::from_millis(1000);

enum TaskState {
    InProgress,
    Successful,
    Failed,
}

// this is a state machine.
// but need to ensure the coherence of the whole swarm.
enum NodeState {
    Free,
    ParentLocked,
    TaskReceived,
    TaskExcecuting(TaskState),
}

struct Node {
    desc: NodeDesc,
    details: NodeDetails,
    last_heard: Instant,
    locked_child: bool,
}

pub struct NodeManager {
    conf: Rc<AstroConf>,
    nid: Nid,
    state: NodeState,
    parent: Option<Node>,  // need backup ids (indirect upper nodes / sibling nodes)
    children: Vec<Node>,
    node_lost_duration: Duration,
    conn_msg_duration: Duration,
    last_conn_msg_t: Instant,
    p: PosVec,
    v: Velocity,
}

impl NodeManager {
    pub fn new_root_node(conf: &Rc<AstroConf>, p: &PosVec, v: &Velocity) -> NodeManager {
        NodeManager {
            conf: conf.clone(),
            nid: vec![conf.id],
            state: NodeState::Free,
            parent: None,
            children: vec![],
            node_lost_duration: DEFAULT_NODE_LOST_DURATION,
            conn_msg_duration: DEFAULT_CONNECTION_MSG_DURATION,
            last_conn_msg_t: Instant::now(),
            p: *p,
            v: *v,
        }
    }

    #[inline]
    pub fn get_id(&self) -> u32 { id_of(&self.nid) }

    #[inline]
    pub fn get_root_id(&self) -> u32 { root_id_of(&self.nid) }

    #[inline]
    pub fn is_root_node(&self) -> bool { self.parent.is_none() }

    #[inline]
    pub fn has_parent(&self) -> bool { self.parent.is_some() }

    #[inline]
    pub fn has_children(&self) -> bool { !self.children.is_empty() }

    #[inline]
    pub fn has_connections(&self) -> bool { self.has_parent() || self.has_children() }

    pub fn has_parent_of_id(&self, id_other: u32) -> bool {
        self.parent.as_ref().is_some_and(|pnd| id_of(&pnd.desc.nid) == id_other)
    }

    pub fn has_child_of_id(&self, id_other: u32) -> bool {
        self.children.iter().any(|cnd| id_of(&cnd.desc.nid) == id_other)
    }

    pub fn has_parent_or_child_of_id(&self, id_other: u32) -> bool {
        self.has_parent_of_id(id_other) || self.has_child_of_id(id_other)
    }

    pub fn get_subswarm_size(&self) -> u32 {
        let mut subswarm_size: u32 = 1;
        for nd in &self.children {
            subswarm_size += nd.details.subswarm;
        }
        subswarm_size
    }

    pub fn get_swarm_size(&self) -> u32 {
        match &self.parent {
            None => self.get_subswarm_size(),
            Some(nd) => nd.desc.swm,
        }
    }

    pub fn generate_node_desc(&self) -> NodeDesc {
        NodeDesc {
            nid: self.nid.clone(),
            p: self.p,
            v: self.v,
            swm: self.get_swarm_size(),
        }
    }

    pub fn generate_node_details(&self) -> NodeDetails {
        NodeDetails {
            subswarm: self.get_subswarm_size(),
            free: match self.state {
                NodeState::Free => true,
                _ => false,
            },
        }
    }

    pub fn update_node(&mut self, p: &PosVec, v: &Velocity,
                       rm: &Vec<u32>, msgs: &Vec<&Msg>, neighbours: &Vec<&NodeDesc>)
    -> (Velocity, Vec<Msg>) {
        self.p = *p;
        self.v = *v;
        self.remove_no_contact_nodes(rm);

        let now = Instant::now();
        let mut msgs_out: Vec<Msg> = vec![];
        for msg in msgs {
            if let Some(mut msgs_to_send) = self.process_msg(now, msg) {
                msgs_out.append(&mut msgs_to_send);
            }
        }
        self.remove_no_connection_nodes(now);

        if let Some(mut msgs_to_send) = self.explore_neighbours(now, neighbours) {
            msgs_out.append(&mut msgs_to_send);
        }

        if let Some(msg_to_send) = self.maybe_generate_connection_msg(now) {
            msgs_out.push(msg_to_send);
        }

        (self.calc_next_v(), msgs_out)
    }

    fn remove_no_contact_nodes(&mut self, rm: &Vec<u32>) {
        if self.parent.as_ref().is_some_and(|pnd| rm.contains(&id_of(&pnd.desc.nid))) {
            self.remove_parent();
        }
        for cid in rm {
            self.remove_child_with_id(*cid);
        }
    }

    fn process_msg(&mut self, now: Instant, msg: &Msg) -> Option<Vec<Msg>> {
        let desc_sdr = &msg.sender;
        match &msg.body {
            MsgBody::BROADCASTING => None,
            MsgBody::CONNECTION(dtl) => { self.update_connection(desc_sdr, dtl, now); None },
            MsgBody::JOIN(dtl) => { self.try_add_child(desc_sdr, dtl, now); None },
            MsgBody::LEAVE => { self.remove_child_with_id(id_of(&desc_sdr.nid)); None },
            MsgBody::TASK(..) => None,
        }
    }

    fn remove_no_connection_nodes(&mut self, now: Instant) {
        if let Some(nd) = &self.parent {
            if now - nd.last_heard > self.node_lost_duration {
                self.remove_parent();
            }
        }
        let rm: Vec<u32> = self.children.iter().filter(
            |nd| now - nd.last_heard > self.node_lost_duration
        ).map(|nd| id_of(&nd.desc.nid)).collect();
        for cid in rm {
            self.remove_child_with_id(cid);
        }
    }

    fn explore_neighbours(&mut self, now: Instant, neighbours: &Vec<&NodeDesc>)
    -> Option<Vec<Msg>> {
        match self.state {
            NodeState::Free => {
                if let Some(m) = self.join_other_tree(neighbours, now) {
                    Some(vec![m])
                } else {
                    None
                }
            },
            _ => None,
        }
    }

    // TODO: this is a bad algorithm, and too long a function.
    fn join_other_tree(&mut self, neighbours: &Vec<&NodeDesc>, now: Instant) -> Option<Msg> {
        // TODO: freshness of the nodes when get nearby targets?
        // TODO: the number of children also needs considered
        let root_self = self.get_root_id();
        let mut candidates: Vec<&NodeDesc> = neighbours.iter().map(|r| *r).filter(
            |desc| root_id_of(&desc.nid) != root_self).collect();
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
                let swarm_size = self.get_swarm_size();
                let mut rng = rand::thread_rng();
                if desc.swm < swarm_size {
                    None
                } else if desc.swm == swarm_size && self.get_root_id() <= root_id_of(&desc.nid) {
                    None
                } else if !self.is_root_node() && rng.gen_range(0..10) < 5 {  // TODO: try to avoid random number
                    None
                } else if self.set_parent(desc, now) {  // TODO: wait for swarm_size to update
                    Some(Msg {
                        sender: self.generate_node_desc(),
                        to_ids: vec![id_of(&desc.nid)],
                        body: MsgBody::JOIN(self.generate_node_details()),
                    })
                } else {
                    None
                }
            },
        }
    }

    fn maybe_generate_connection_msg(&mut self, now: Instant) -> Option<Msg> {
        if now - self.last_conn_msg_t > self.conn_msg_duration && self.has_connections() {
            self.last_conn_msg_t = now;
            Some(self.generate_connection_msg())
        } else {
            None
        }
    }

    fn generate_connection_msg(&self) -> Msg {
        let mut to_ids: Vec<u32> = vec![];
        if let Some(nd) = &self.parent {
            to_ids.push(id_of(&nd.desc.nid));
        }
        let mut cids: Vec<u32> = vec![];
        for nd in &self.children {
            to_ids.push(id_of(&nd.desc.nid));
            cids.push(id_of(&nd.desc.nid));
        }
        Msg {
            sender: self.generate_node_desc(),
            to_ids,
            body: MsgBody::CONNECTION(self.generate_node_details()),
        }
    }

    fn calc_next_v(&self) -> Velocity {
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

    fn update_connection(&mut self, desc: &NodeDesc, dtl: &NodeDetails, now: Instant) {
        if self.has_parent_of_id(id_of(&desc.nid)) {
            // the description is from recognised parent
            self.try_update_or_create_parent_connection(desc, dtl, now);
            // if the other node is not a valid parent, update will not be carried out,
            // just wait the parent to timeout and be removed.
        } else {
            self.update_child_connection(desc, dtl, now);
        }
    }

    // TODO: modify this function and the add child function, similar to those with parent
    fn update_child_connection(&mut self, desc: &NodeDesc, dtl: &NodeDetails, now: Instant) {
        let id = self.get_id();
        let id_other = id_of(&desc.nid);
        for cnd in &mut self.children {
            let cid = id_of(&cnd.desc.nid);
            if cid == id_other {
                // the description is from a recognised child
                if valid_descendant_of(id_other, &self.nid)
                    && parent_id_of(&desc.nid).is_some_and(|v| v == id) {
                    // valid child,
                    // and the child recognised this node as its parent.
                    cnd.desc = desc.clone();
                    cnd.details = dtl.clone();
                    cnd.last_heard = now;
                }  // otherwise, just wait the child to timeout and be removed
                return;
            }
        }
    }

    fn try_add_child(&mut self, desc: &NodeDesc, dtl: &NodeDetails, now: Instant) {
        let id_other = id_of(&desc.nid);
        if self.has_parent_or_child_of_id(id_other) {
            // the other node is already a recognised child
            return;
        }
        if !valid_descendant_of(id_other, &self.nid) {
            // the other node is not a valid child of this node
            return;
        }
        self.children.push(Node {
            desc: desc.clone(),
            details: dtl.clone(),
            last_heard: now,
            locked_child: false,
        })
    }

    fn set_parent(&mut self, desc: &NodeDesc, now: Instant) -> bool {
        let dtl = NodeDetails {
            subswarm: 0,
            free: true,
        };
        self.try_update_or_create_parent_connection(desc, &dtl, now)
    }

    // TODO: split this function, move creation part into set_parent
    fn try_update_or_create_parent_connection(&mut self, desc: &NodeDesc, dtl: &NodeDetails, now: Instant) -> bool {
        let id = self.get_id();
        if !valid_descendant_of(id, &desc.nid) {
            // the other node is not a valid parent of this node
            false
        } else {
            match &mut self.parent {
                None => {
                    self.parent = Some(Node {
                        desc: desc.clone(),
                        details: dtl.clone(),
                        last_heard: now,
                        locked_child: false,
                    });
                    println!("new connection: {:?} <- {}", desc.nid, id);
                },
                Some(pnd) => {
                    pnd.desc = desc.clone();
                    pnd.details = dtl.clone();
                    pnd.last_heard = now;
                },
            }
            self.nid = desc.nid.clone();
            self.nid.push(id);
            if dtl.free {
                self.state = NodeState::Free;
            }
            true
        }
    }

    fn remove_parent(&mut self) {
        let id = self.get_id();
        self.parent = None;
        self.nid = vec![id];
        self.state = NodeState::Free;
    }

    fn remove_child_with_id(&mut self, cid: u32) {
        for (idx, cnd) in self.children.iter().enumerate() {
            if id_of(&cnd.desc.nid) == cid {
                self.remove_child_with_idx(idx);
                break;
            }
        }
    }

    fn remove_child_with_idx(&mut self, idx: usize) {
        let cnd = self.children.remove(idx);
        if cnd.locked_child {
            self.state = NodeState::TaskExcecuting(TaskState::Failed);
        }
    }
}