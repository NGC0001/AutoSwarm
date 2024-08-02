use std::cmp::Ordering;
use std::option::Option;
use std::rc::Rc;
use std::time::{Duration, Instant};

use super::super::astroconf::AstroConf;
use super::super::kinetics::{distance, PosVec, Velocity};
use super::contacts::Contact;
use super::msg::{Nid, id_of, root_id_of, is_id_valid_descendant_of};
use super::msg::{NodeDesc, NodeDetails, MsgBody, Msg};

pub const DEFAULT_NODE_LOST_DURATION: Duration = Duration::from_secs(5);
pub const DEFAULT_CONNECTION_MSG_DURATION: Duration = Duration::from_millis(1000);
pub const NEW_PARENT_FRESHNESS: Duration = Duration::from_millis(1000);
pub const CHILD_ADDING_TIMESCALE: Duration = Duration::from_millis(1000);
pub const CHILD_ADDING_RATE_LIMIT: f32 = 2.0;

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

impl Node {
    #[inline]
    pub fn get_id(&self) -> u32 { self.desc.get_id() }
}

pub struct NodeManager {
    now: Instant,
    child_adding_rate: f32,
    p: PosVec,
    v: Velocity,

    conf: Rc<AstroConf>,
    nid: Nid,
    state: NodeState,
    parent: Option<Node>,  // need backup ids (indirect upper nodes / sibling nodes)
    children: Vec<Node>,
    node_lost_duration: Duration,
    conn_msg_duration: Duration,
    last_conn_msg_t: Instant,
}

impl NodeManager {
    pub fn new_root_node(conf: &Rc<AstroConf>, p: &PosVec, v: &Velocity) -> NodeManager {
        let now = Instant::now();
        NodeManager {
            now,
            child_adding_rate: 0.0,
            p: *p,
            v: *v,

            conf: conf.clone(),
            nid: vec![conf.id],
            state: NodeState::Free,
            parent: None,
            children: vec![],
            node_lost_duration: DEFAULT_NODE_LOST_DURATION,
            conn_msg_duration: DEFAULT_CONNECTION_MSG_DURATION,
            last_conn_msg_t: now,
        }
    }

    #[inline]
    pub fn get_id(&self) -> u32 { id_of(&self.nid) }

    #[inline]
    pub fn get_root_id(&self) -> u32 { root_id_of(&self.nid) }

    #[inline]
    pub fn is_root_node(&self) -> bool { self.parent.is_none() }

    #[inline]
    pub fn is_valid_ancestor_of(&self, id: u32) -> bool { is_id_valid_descendant_of(id, &self.nid) }

    #[inline]
    pub fn has_parent(&self) -> bool { self.parent.is_some() }

    #[inline]
    pub fn has_children(&self) -> bool { !self.children.is_empty() }

    #[inline]
    pub fn has_connections(&self) -> bool { self.has_parent() || self.has_children() }

    pub fn has_parent_of_id(&self, id_other: u32) -> bool {
        self.parent.as_ref().is_some_and(|pnd| pnd.get_id() == id_other)
    }

    pub fn has_child_of_id(&self, id_other: u32) -> bool {
        self.children.iter().any(|cnd| cnd.get_id() == id_other)
    }

    pub fn has_task(&self) -> bool {
        match self.state {
            NodeState::TaskReceived => true,
            NodeState::TaskExcecuting(..) => true,
            _ => false,
        }
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
            tsk: self.has_task(),
        }
    }

    pub fn generate_node_details(&self) -> NodeDetails {
        NodeDetails {
            subswarm: self.get_subswarm_size(),
        }
    }

    pub fn update_node(&mut self, p: &PosVec, v: &Velocity,
                       rm: &Vec<u32>, msgs: &Vec<&Msg>, neighbours: &Vec<&Contact>)
    -> (Velocity, Vec<Msg>) {
        let previous = self.now;
        self.now = Instant::now();
        self.child_adding_rate *= ((self.now - previous).as_secs_f32() / CHILD_ADDING_TIMESCALE.as_secs_f32()).exp();
        self.p = *p;
        self.v = *v;
        // messages may be lost, so in most cases they should carry state, rather than carry events.
        // if an event message is not received by all parties involved, its (partial) effect should be revocable.
        let mut msgs_out: Vec<Msg> = vec![];

        self.remove_no_contact_nodes(rm);  // contact-losing events
        for msg in msgs {  // message events
            msgs_out.append(&mut self.process_msg(msg, neighbours));
        }
        self.remove_no_connection_nodes();  // connection-losing events

        msgs_out.append(&mut self.explore_neighbourhood(neighbours));  // neighbourhood events
        // TODO:
        // self.manage_state();
        // msgs_out.append(self.maybe_generate_state_msg());
        if let Some(msg_to_send) = self.maybe_generate_connection_msg() {
            msgs_out.push(msg_to_send);
        }
        (self.calc_next_v(), msgs_out)
    }

    fn remove_no_contact_nodes(&mut self, rm: &Vec<u32>) {
        if self.parent.as_ref().is_some_and(|pnd| rm.contains(&pnd.get_id())) {
            self.remove_parent();
        }
        for cid in rm {
            self.remove_child_of_id(*cid);
        }
    }

    fn process_msg(&mut self, msg: &Msg, neighbours: &Vec<&Contact>) -> Vec<Msg> {
        let desc_sdr = &msg.sender;
        let mut msg_out: Vec<Msg> = vec![];
        match &msg.body {
            MsgBody::Broadcasting => (),
            MsgBody::Connection(dtl) => self.update_connection(desc_sdr, dtl),

            MsgBody::Join(dtl) => msg_out.push(self.add_child_or_reject(desc_sdr, dtl)),
            MsgBody::Accept => (),
            MsgBody::Reject => self.remove_parent_of_id(desc_sdr.get_id()),

            MsgBody::Leave => self.remove_child_of_id(desc_sdr.get_id()),

            MsgBody::ChangeParent(pid_new) => msg_out.append(&mut self.change_parent(*pid_new, neighbours)),

            MsgBody::Task(..) => (),
        }
        msg_out
    }

    fn remove_no_connection_nodes(&mut self) {
        if let Some(nd) = &self.parent {
            if self.now - nd.last_heard > self.node_lost_duration {
                self.remove_parent();
            }
        }
        let rm: Vec<u32> = self.children.iter().filter(
            |nd| self.now - nd.last_heard > self.node_lost_duration
        ).map(|nd| nd.get_id()).collect();
        for cid in rm {
            self.remove_child_of_id(cid);
        }
    }

    fn explore_neighbourhood(&mut self, neighbours: &Vec<&Contact>) -> Vec<Msg> {
        let mut msg_out: Vec<Msg> = vec![];
        if let NodeState::Free = self.state {
            if let Some(m) = self.try_join_other_swarm(neighbours) {
                msg_out.push(m);
            }
        }
        msg_out
    }

    fn try_join_other_swarm(&mut self, neighbours: &Vec<&Contact>) -> Option<Msg> {
        let desc_self = self.generate_node_desc();
        match self.find_parent_candidate(&desc_self, neighbours) {
            None => None,
            Some(candidate) => self.set_parent_and_create_join_msg(candidate),
        }
    }

    fn find_parent_candidate<'a, 'b, 'c>(&self, desc_self: &'a NodeDesc, neighbours: &Vec<&'b Contact>)
    -> Option<&'c NodeDesc> where 'a: 'c, 'b: 'c {
        let root_id_self = self.get_root_id();
        let mut candidates: Vec<&NodeDesc> = neighbours.iter().filter(
            |t| self.now - t.last_heard < NEW_PARENT_FRESHNESS  // freshness of candidate
        ).map(|t| &t.desc).filter(
            |nd| !nd.tsk && nd.get_root_id() != root_id_self  // no task, in different swarm
        ).collect();
        candidates.push(&desc_self);
        candidates.sort_unstable_by(|desc1, desc2| {
            let cmp_swm = desc2.swm.cmp(&desc1.swm);  // bigger swarm size
            if cmp_swm != Ordering::Equal { return cmp_swm; }
            let cmp_root_id = desc1.get_root_id().cmp(&desc2.get_root_id());  // smaller root id
            if cmp_root_id != Ordering::Equal { return cmp_root_id; }
            let cmp_dist = distance(&desc1.p, &self.p).partial_cmp(
                &distance(&desc2.p, &self.p)).unwrap();  // closer node
            cmp_dist
            // may also take into account the number of children, but NodeDesc doesn't carry this info
            // may also take into account the depth(rank) of the node on the swarm tree
        });
        let candidate: &NodeDesc = candidates.first().unwrap();
        if candidate.get_id() == desc_self.get_id() {  // all other swarms are worse than the current swarm
            None
        } else {
            Some(candidate)
        }
    }

    fn maybe_generate_connection_msg(&mut self) -> Option<Msg> {
        if self.now - self.last_conn_msg_t > self.conn_msg_duration && self.has_connections() {
            self.last_conn_msg_t = self.now;
            Some(self.generate_connection_msg())
        } else {
            None
        }
    }

    fn generate_connection_msg(&self) -> Msg {
        let mut to_ids: Vec<u32> = vec![];
        if let Some(nd) = &self.parent {
            to_ids.push(nd.get_id());
        }
        let mut cids: Vec<u32> = vec![];
        for nd in &self.children {
            let cid = nd.get_id();
            to_ids.push(cid);
            cids.push(cid);
        }
        Msg {
            sender: self.generate_node_desc(),
            to_ids,
            body: MsgBody::Connection(self.generate_node_details()),
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

    fn update_connection(&mut self, desc: &NodeDesc, dtl: &NodeDetails) {
        if !self.has_parent_of_id(desc.get_id()) {
            // the description may be from a child
            self.update_child_connection(desc, dtl);
        } else {
            // the description is from recognised parent
            let id = self.get_id();
            if desc.is_valid_ancestor_of(id) {  // valid parent
                let pnd = self.parent.as_mut().unwrap();
                pnd.desc = desc.clone();
                pnd.details = dtl.clone();
                pnd.last_heard = self.now;
                self.on_parent_info_updated();
            }  // otherwise, just wait the parent to timeout and be removed
        }
    }

    fn update_child_connection(&mut self, desc: &NodeDesc, dtl: &NodeDetails) {
        let id = self.get_id();
        let id_other = desc.get_id();
        for cnd in &mut self.children {
            let cid = cnd.get_id();
            if cid == id_other {
                // the description is from a recognised child
                if is_id_valid_descendant_of(id_other, &self.nid)
                    && desc.get_parent_id().is_some_and(|v| v == id) {
                    // valid child,
                    // and the child recognised this node as its parent.
                    cnd.desc = desc.clone();
                    cnd.details = dtl.clone();
                    cnd.last_heard = self.now;
                }  // otherwise, just wait the child to timeout and be removed
                return;
            }
        }
    }

    fn add_child_or_reject(&mut self, desc: &NodeDesc, dtl: &NodeDetails) -> Msg {
        let id_other = desc.get_id();
        let msg_body: MsgBody;
        if !self.is_valid_ancestor_of(id_other)  // not valid child
            || self.has_task()  // has task, won't accept child
            || self.child_adding_rate > CHILD_ADDING_RATE_LIMIT {  // number of children increases too fast
            msg_body = MsgBody::Reject;
        } else {
            msg_body = MsgBody::Accept;
            if !self.has_child_of_id(id_other) {
                self.children.push(Node {
                    desc: desc.clone(),
                    details: dtl.clone(),
                    last_heard: self.now,
                    locked_child: false,
                });
                self.child_adding_rate += 1.0;
            }  // else, the other node is already a recognised child
        }
        Msg {
            sender: self.generate_node_desc(),
            to_ids: vec![id_other],
            body: msg_body,
        }
    }

    fn change_parent(&mut self, pid_new: u32, neighbours: &Vec<&Contact>) -> Vec<Msg> {
        let mut out: Vec<Msg> = vec![];
        self.remove_parent();
        for t in neighbours {
            if t.desc.get_id() == pid_new {
                if let Some(m) = self.set_parent_and_create_join_msg(&t.desc) {
                    out.push(m);
                }
                break;
            }
        }
        out
    }

    fn set_parent_and_create_join_msg(&mut self, desc: &NodeDesc) -> Option<Msg> {
        if self.set_parent(desc) {
            Some(Msg {
                sender: self.generate_node_desc(),
                to_ids: vec![desc.get_id()],
                body: MsgBody::Join(self.generate_node_details()),
            })
        } else {
            None
        }
    }

    fn set_parent(&mut self, desc: &NodeDesc) -> bool {
        let id = self.get_id();
        if !self.is_valid_ancestor_of(id) {
            false
        } else {  // valid parent
            self.parent = Some(Node {
                desc: desc.clone(),
                details: NodeDetails {
                    subswarm: 0,
                },
                last_heard: self.now,
                locked_child: false,
            });
            self.on_parent_info_updated();
            println!("new connection: {:?} <- {}", desc.nid, id);
            true
        }
    }

    fn remove_parent_of_id(&mut self, pid: u32) {
        if self.has_parent_of_id(pid) {
            self.remove_parent();
        }
    }

    fn remove_parent(&mut self) {
        self.parent = None;
        self.on_parent_info_updated();
    }

    fn on_parent_info_updated(&mut self) {
        let id = self.get_id();
        match &self.parent {
            None => {
                self.nid = vec![id];
                self.state = NodeState::Free;
            },
            Some(pnd) => {
                self.nid = pnd.desc.nid.clone();
                self.nid.push(id);
                if !pnd.desc.tsk {
                    self.state = NodeState::Free;
                }
            },
        };
    }

    fn remove_child_of_id(&mut self, cid: u32) {
        for (idx, cnd) in self.children.iter().enumerate() {
            if cnd.get_id() == cid {
                self.remove_child_of_idx(idx);
                break;
            }
        }
    }

    fn remove_child_of_idx(&mut self, idx: usize) {
        let cnd = self.children.remove(idx);
        if cnd.locked_child {
            self.state = NodeState::TaskExcecuting(TaskState::Failed);
        }
    }
}