use std::cmp::Ordering;
use std::option::Option;
use std::rc::Rc;
use std::time::{Duration, Instant};

use super::super::astroconf::AstroConf;
use super::super::kinetics::{distance, PosVec, Velocity};
use super::contacts::Contact;
use super::msg::{id_of, is_id_valid_descendant_of, parent_id_of, root_id_of, Nid};
use super::msg::{NodeDesc, NodeDetails, JoinAppl, AssignChildAppl, Task, MsgBody, Msg};
use super::tm::TaskManager;

pub const DEFAULT_NODE_LOST_DURATION: Duration = Duration::from_secs(5);
pub const DEFAULT_CONNECTION_MSG_DURATION: Duration = Duration::from_millis(200);
pub const NEW_PARENT_FRESHNESS: Duration = Duration::from_millis(1000);
pub const CHILD_ADDING_TIMESCALE: Duration = Duration::from_millis(300);
pub const CHILD_ADDING_RATE_LIMIT: f32 = 0.5;

// TaskState::Success means all subnodes and this node have received and succeeded task tid,
//     this indicates that the subswarm has succeeded task tid.
// TaskState::Failure means any subnode or this node has received and failed task tid,
//     this indicates that the subswarm has failed task tid.
// TaskState::InProgress means this node has received task tid, but hasn't succeeded or failed yet,
//     in this node state, all subnodes may or may not have received task tid.
#[derive(Copy, Clone)]
pub enum TaskState {
    InProgress,
    Success,
    Failure,
}

// this is a state machine of the node.
// but need to ensure the coherence of the whole swarm.
enum NodeState {
    Free,
    InTask(u32, TaskState),
}

struct Node {
    desc: NodeDesc,
    details: NodeDetails,
    last_heard: Instant,
}

impl Node {
    #[inline]
    pub fn get_id(&self) -> u32 { self.desc.get_id() }
}

pub struct NodeManager {
    conf: Rc<AstroConf>,
    now: Instant,
    child_adding_rate: f32,
    p: PosVec,
    v: Velocity,

    nid: Nid,
    state: NodeState,
    tm: TaskManager,

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
            conf: conf.clone(),
            now,
            child_adding_rate: 0.0,
            p: *p,
            v: *v,

            nid: vec![conf.id],
            state: NodeState::Free,
            tm: TaskManager::new(),

            parent: None,
            children: vec![],
            node_lost_duration: DEFAULT_NODE_LOST_DURATION,
            conn_msg_duration: DEFAULT_CONNECTION_MSG_DURATION,
            last_conn_msg_t: now,
        }
    }

    #[inline]
    pub fn get_nid(&self) -> &Vec<u32> { &self.nid }

    #[inline]
    pub fn get_id(&self) -> u32 { id_of(self.get_nid()) }

    #[inline]
    pub fn get_root_id(&self) -> u32 { root_id_of(self.get_nid()) }

    #[inline]
    pub fn is_root_node(&self) -> bool { !self.has_parent() }

    #[inline]
    pub fn get_parent_id(&self) -> Option<u32> { parent_id_of(self.get_nid()) }

    #[inline]
    pub fn is_valid_ancestor_of(&self, id: u32) -> bool { is_id_valid_descendant_of(id, self.get_nid()) }

    #[inline]
    pub fn is_valid_descendant_of(&self, desc: &NodeDesc) -> bool {
        desc.is_valid_ancestor_of(self.get_id())
    }

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

    #[inline]
    pub fn is_free(&self) -> bool { !self.has_task() }

    pub fn has_task(&self) -> bool {
        match self.state {
            NodeState::InTask(..) => true,
            NodeState::Free => false,
        }
    }

    pub fn has_task_of_id(&self, id: u32) -> bool {
        match self.state {
            NodeState::InTask(tid, _) => tid == id,
            NodeState::Free => false,
        }
    }

    pub fn get_task_id(&self) -> Option<u32> {
        match self.state {
            NodeState::InTask(tid, _) => Some(tid),
            NodeState::Free => None,
        }
    }

    pub fn get_subswarm_task_state(&self) -> Option<(u32, TaskState)> {
        match self.state {
            NodeState::Free => None,
            NodeState::InTask(tid, TaskState::Success) => Some((tid, TaskState::Success)),
            NodeState::InTask(tid, TaskState::Failure) => Some((tid, TaskState::Failure)),
            NodeState::InTask(tid, TaskState::InProgress) => {
                if self.children.iter().all(|cnd| cnd.details.has_subswm_tsk_id(tid)) {
                    Some((tid, TaskState::InProgress))
                } else {
                    None
                }
            },
        }
    }

    pub fn is_subswarm_in_task(&self) -> bool {
        match self.state {
            NodeState::InTask(tid, _) => {
                self.children.iter().all(|cnd| cnd.details.has_subswm_tsk_id(tid))
            },
            NodeState::Free => false,
        }
    }

    pub fn get_subswarm_size(&self) -> u32 {
        1 + self.children.iter().map(|cnd| cnd.details.subswarm).sum::<u32>()
    }

    pub fn get_swarm_size(&self) -> u32 {
        match &self.parent {
            None => self.get_subswarm_size(),
            Some(pnd) => pnd.desc.swm,
        }
    }

    pub fn generate_node_desc(&self) -> NodeDesc {
        NodeDesc {
            nid: self.nid.clone(),
            p: self.p,
            v: self.v,
            swm: self.get_swarm_size(),
            tsk: self.get_task_id(),
        }
    }

    pub fn generate_node_details(&self) -> NodeDetails {
        NodeDetails {
            subswarm: self.get_subswarm_size(),
            subswm_tsk: self.get_subswarm_task_state(),
        }
    }

    pub fn update_node(&mut self, p: &PosVec, v: &Velocity,
                       rm: &Vec<u32>, msgs: &Vec<&Msg>, neighbours: &Vec<&Contact>)
    -> (Velocity, Vec<Msg>) {
        let previous = self.now;
        self.now = Instant::now();
        self.child_adding_rate *= (-(self.now - previous).as_secs_f32() / CHILD_ADDING_TIMESCALE.as_secs_f32()).exp();
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

        msgs_out.append(&mut self.manage_node_state(neighbours));

        self.maybe_generate_connection_msg(&mut msgs_out);
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
            MsgBody::Empty => (),
            MsgBody::Connection(dtl) => self.update_connection(desc_sdr, dtl),

            MsgBody::Join(appl) => msg_out.push(self.add_child_or_reject(desc_sdr, appl)),
            MsgBody::Accept => (),
            MsgBody::Reject => self.remove_parent_of_id(desc_sdr.get_id()),
            MsgBody::Leave => self.remove_child_of_id(desc_sdr.get_id()),

            MsgBody::ChangeParent(pid_new) => self.change_parent(*pid_new, neighbours),
            MsgBody::AssignChild(appl) => self.add_assigned_child(appl, neighbours),

            MsgBody::Task(task) => msg_out.append(&mut self.relay_or_accept_task(task)),
        }
        msg_out
    }

    fn remove_no_connection_nodes(&mut self) {
        if let Some(pnd) = &self.parent {
            if self.now - pnd.last_heard > self.node_lost_duration {
                self.remove_parent();
            }
        }
        let rm: Vec<u32> = self.children.iter().filter(
            |cnd| self.now - cnd.last_heard > self.node_lost_duration
        ).map(|cnd| cnd.get_id()).collect();
        for cid in rm {
            self.remove_child_of_id(cid);
        }
    }

    fn manage_node_state(&mut self, neighbours: &Vec<&Contact>) -> Vec<Msg> {
        // child nodes generally follow the state of their parent,
        // but they themselves decide whether their subtask is success/failure.
        if let NodeState::InTask(_, TaskState::InProgress) = self.state {
            self.update_task_in_progress();
        }

        if self.is_root_node() {
            self.manage_root_node_state();  // manages the overall state of the whole swarm
        }

        if self.is_free() {
            self.try_join_other_swarm(neighbours)
        } else {
            self.generate_task_msgs()
        }
    }

    fn update_task_in_progress(&mut self) {
        match self.tm.get_current_task() {
            None => (),  // subtask not received yet
            Some(t) => {
                if self.is_subswarm_in_task() {
                    self.tm.execute_task();
                }
                if self.tm.is_task_failure() || self.children.iter().any(|cnd| cnd.details.failure) {
                    self.switch_state_to_in_task(t.id, TaskState::Failure);
                }
                if self.tm.is_task_success() && self.children.iter().all(|cnd| cnd.details.success) {
                    self.switch_state_to_in_task(t.id, TaskState::Success);
                }
            },
        }
    }

    // this function is only run by root node
    fn manage_root_node_state(&mut self) {
        if let NodeState::InTask(tid, ts) = self.state {
            match ts {
                TaskState::InProgress => { return; },  // do nothing
                TaskState::Success => { println!("task {} succeeded", tid); },
                TaskState::Failure => { println!("task {} failed", tid); },
            }
            // task failure, or task success
        }
        // free, or task failure, or task success
        match self.tm.pop_queued_task() {
            Some(t) => {  // swarm from free/task to another task
                self.switch_state_to_in_task(t.id, TaskState::InProgress);
                self.tm.set_current_task(t);
            },
            None => {  // swarm from task/free to free
                self.switch_state_to_free();
            },
        }
    }

    fn try_join_other_swarm(&mut self, neighbours: &Vec<&Contact>) -> Vec<Msg> {
        let mut msgs: Vec<Msg> = vec![];
        let desc_self = self.generate_node_desc();
        if let Some(candidate) = self.find_parent_candidate(&desc_self, neighbours) {
            let src_tree: u32 = self.get_root_id();
            let prev_parent = self.get_parent_id();
            if self.set_parent(candidate) {
                msgs.push(Msg {
                    sender: self.generate_node_desc(),
                    to_ids: vec![candidate.get_id()],
                    body: MsgBody::Join(JoinAppl {
                        dtl: self.generate_node_details(),
                        src_tree,
                    }),
                });
                if let Some(prev_pid) = prev_parent {
                    msgs.push(Msg {
                        sender: self.generate_node_desc(),
                        to_ids: vec![prev_pid],
                        body: MsgBody::Leave,
                    });
                }
            }
        }
        msgs
    }

    fn find_parent_candidate<'a, 'b, 'c>(&self, desc_self: &'a NodeDesc, neighbours: &Vec<&'b Contact>)
    -> Option<&'c NodeDesc> where 'a: 'c, 'b: 'c {
        let root_id_self = self.get_root_id();
        let mut candidates: Vec<&NodeDesc> = neighbours.iter().filter(
            |t| self.now - t.last_heard < NEW_PARENT_FRESHNESS  // freshness of candidate
        ).map(|t| &t.desc).filter(
            |nd| nd.is_free() && nd.get_root_id() != root_id_self  // no task, in different swarm
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

    fn maybe_generate_connection_msg(&mut self, msgs_out: &mut Vec<Msg>) {
        // TODO:
        // connection message should be sent periodically, or immediately if node status changes.
        // however currently node status changes cannot be detected.
        // instead, check whether `msgs_out` isn't empty, which indicates potential status change.
        if self.has_connections() && (
            !msgs_out.is_empty() || self.now - self.last_conn_msg_t > self.conn_msg_duration) {
            msgs_out.push(self.generate_connection_msg());
            self.last_conn_msg_t = self.now;
        }
    }

    fn generate_connection_msg(&self) -> Msg {
        let mut to_ids: Vec<u32> = self.children.iter().map(|cnd| cnd.get_id()).collect();
        if let Some(pnd) = &self.parent {
            to_ids.push(pnd.get_id());
        }
        assert!(!to_ids.is_empty());
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
                if dist < self.conf.msg_range / 2.0 {
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
        if self.has_parent_of_id(desc.get_id()) {
            // the description is from recognised parent
            if self.is_valid_descendant_of(desc) {  // valid parent
                let pnd = self.parent.as_mut().unwrap();
                pnd.desc = desc.clone();
                pnd.details = dtl.clone();
                pnd.last_heard = self.now;
                self.on_parent_info_updated();
            }  // otherwise, just wait the parent to timeout and be removed
        } else {
            // the description may be from a child
            self.update_child_connection(desc, dtl);
        }
    }

    fn update_child_connection(&mut self, desc: &NodeDesc, dtl: &NodeDetails) {
        let id = self.get_id();
        let id_other = desc.get_id();
        if let Some(idx) = self.children.iter().position(|cnd| cnd.get_id() == id_other) {
            // the description is from a recognised child
            if self.is_valid_ancestor_of(id_other) && desc.has_parent_of_id(id) {
                // valid child,
                // and the child recognised this node as its parent.
                let cnd: &mut Node = &mut self.children[idx];
                cnd.desc = desc.clone();
                cnd.details = dtl.clone();
                cnd.last_heard = self.now;
            }  // otherwise, just wait the child to timeout and be removed
        }
    }

    // deal with Join, but not with ChangeParent/AssignChild
    fn add_child_or_reject(&mut self, desc: &NodeDesc, appl: &JoinAppl) -> Msg {
        let id_other = desc.get_id();
        let accept = self.is_valid_ancestor_of(id_other) && self.get_root_id() != appl.src_tree
            && self.is_free() && self.child_adding_rate < CHILD_ADDING_RATE_LIMIT;
        if accept && !self.has_child_of_id(id_other) {
            self.children.push(Node {
                desc: desc.clone(),
                details: appl.dtl.clone(),
                last_heard: self.now,
            });
            self.child_adding_rate += 1.0;
            println!("new connection: {:?} <- {}", self.get_nid(), id_other);
        }
        Msg {
            sender: self.generate_node_desc(),
            to_ids: vec![id_other],
            body: if accept { MsgBody::Accept } else { MsgBody::Reject },
        }
    }

    fn change_parent(&mut self, pid_new: u32, neighbours: &Vec<&Contact>) {
        let mut changed: bool = false;
        if let Some(t) = neighbours.iter().find(|t| t.desc.get_id() == pid_new) {
            changed = self.set_parent(&t.desc);
        }
        if !changed {
            self.remove_parent();  // node detached from current swarm tree
        }
    }

    fn add_assigned_child(&mut self, appl: &AssignChildAppl, neighbours: &Vec<&Contact>) {
        let mut added: bool = false;
        if let Some(t) = neighbours.iter().find(|t| t.desc.get_id() == appl.cid) {
            if self.is_valid_ancestor_of(appl.cid) && self.get_root_id() == t.desc.get_root_id()
                && self.has_task() && t.desc.has_task_of_id(self.get_task_id().unwrap())
                && !self.has_child_of_id(appl.cid) {
                self.children.push(Node {
                    desc: t.desc.clone(),
                    details: appl.dtl.clone(),
                    last_heard: self.now,
                });
                added = true;
                println!("assigned connection: {:?} <- {}", self.get_nid(), appl.cid);
            }
        }
        if !added {
            self.fail_task();
        }
    }

    fn relay_or_accept_task(&mut self, task: &Task) -> Vec<Msg> {
        match self.get_parent_id() {
            Some(pid) => vec![Msg {  // relay task to parent node
                sender: self.generate_node_desc(),
                to_ids: vec![pid],
                body: MsgBody::Task(task.clone()),
            }],
            None => {  // root node
                self.tm.add_task_if_new(task);
                vec![]
            },
        }
    }

    fn set_parent(&mut self, desc: &NodeDesc) -> bool {
        if !self.is_valid_descendant_of(desc) {
            false
        } else {  // valid parent
            self.parent = Some(Node {
                desc: desc.clone(),
                details: NodeDetails {
                    subswarm: 0,  // value here should not matter
                    subswm_tsk: None,  // value here should not matter
                },
                last_heard: self.now,
            });
            self.on_parent_info_updated();
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
        self.nid = vec![self.get_id()];
        self.switch_state_to_free();
    }

    fn on_parent_info_updated(&mut self) {
        let id = self.get_id();
        let pnd = self.parent.as_ref().unwrap();
        self.nid = pnd.desc.nid.clone();
        self.nid.push(id);
        match pnd.desc.tsk {
            None => { self.switch_state_to_free(); },
            Some(tid) => {
                if !self.has_task_of_id(tid) {
                    // non-root node may re-enter InProgress state from previous InProgress state
                    self.switch_state_to_in_task(tid, TaskState::InProgress);
                }
            },
        };
    }

    // TODO: do not change state if it's rearranging child within same tree
    fn remove_child_of_id(&mut self, cid: u32) {
        if let Some(idx) = self.children.iter().position(|cnd| cnd.get_id() == cid) {
            let cnd = self.children.remove(idx);
            println!("delete connection: {:?} <-x {}", self.get_nid(), cnd.get_id());
            if self.is_subswarm_in_task() && cnd.desc.has_task_of_id(self.get_task_id().unwrap()) {
                self.fail_task();
            }
        }
    }

    fn fail_task(&mut self) {
        match self.state {
            NodeState::Free => (),
            NodeState::InTask(tid, _) => {
                self.switch_state_to_in_task(tid, TaskState::Failure);
            },
        };
    }

    fn switch_state_to_free(&mut self) {
        self.state = NodeState::Free;
        self.tm.clear_current_task();
    }

    fn switch_state_to_in_task(&mut self, tid: u32, ts: TaskState) {
        let mut switched: bool = true;
        if let NodeState::InTask(o_tid, TaskState::InProgress) = self.state {
            if let TaskState::InProgress = ts {
                switched = o_tid != tid;  // the same task in progress, not considered a switch
            }
        }
        self.state = NodeState::InTask(tid, ts);
        if switched {
            self.tm.clear_current_task();
        }
    }
}