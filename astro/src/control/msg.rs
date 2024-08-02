use std::{option::Option, time::Duration};

use serde::{Deserialize, Serialize};

use super::{PosVec, Velocity};

// the node id of a uav in a tree structure.
// it is: id(top) -> id -> ... -> id(this)
pub type Nid = Vec<u32>;

#[inline]
pub fn id_of(nid: &Nid) -> u32 {
    *nid.last().unwrap()
}

#[inline]
pub fn root_id_of(nid: &Nid) -> u32 {
    *nid.first().unwrap()
}

#[inline]
pub fn is_root_node(nid: &Nid) -> bool {
    nid.len() == 1
}

pub fn parent_id_of(nid: &Nid) -> Option<u32> {
    let len = nid.len();
    match len {
        ..=1 => None,
        _ => Some(nid[len - 2]),
    }
}

#[inline]
pub fn is_id_valid_descendant_of(id: u32, p_nid: &Nid) -> bool {
    !p_nid.contains(&id)  // non-cyclic
}

// description of a node in the tree structure.
// need to be generated by that node, passed to its parent and children.
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct NodeDesc {
    pub nid: Nid,  // structural id of node, down-flowing data
    pub p: PosVec,
    pub v: Velocity,
    pub swm: u32,  // the size of the swarm, down-flowing data
    pub tsk: bool,  // whether the node has a task, down-flowing data
}

impl NodeDesc {
    #[inline]
    pub fn get_id(&self) -> u32 { id_of(&self.nid) }

    #[inline]
    pub fn get_root_id(&self) -> u32 { root_id_of(&self.nid) }

    #[inline]
    pub fn is_root_node(&self) -> bool { is_root_node(&self.nid) }

    #[inline]
    pub fn get_parent_id(&self) -> Option<u32> { parent_id_of(&self.nid) }

    #[inline]
    pub fn is_valid_ancestor_of(&self, id: u32) -> bool {
        is_id_valid_descendant_of(id, &self.nid)
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct NodeDetails {
    pub subswarm: u32,  // the size of the subswarm, up-flowing data
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Line {
    pub points: Vec<PosVec>,
    pub closed: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Task {
    pub lines: Vec<Line>,
    pub duration: Duration,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum MsgBody {
    Broadcasting,  // sender reports basic status to all neighbours, no specific receiver
    Connection(NodeDetails),  // sender repports status to receiver (parent and children)

    Join(NodeDetails),  // sender wants to set the receiver as its parent
    Accept,  // sender rejects the receiver as its child
    Reject,  // sender accepts the receiver as its child

    Leave,  // sender stops recognising the receiver as its parent

    ChangeParent(u32),  // sender sets a third node as the receiver's new parent

    Task(Task),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Msg {
    pub sender: NodeDesc,  // node description of message sender
    pub to_ids: Vec<u32>,  // target message receivers, None means broadcasting
    pub body: MsgBody,
}