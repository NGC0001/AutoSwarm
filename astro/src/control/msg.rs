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

#[inline]
pub fn valid_descendant_of(id: u32, p_nid: &Nid) -> bool {
    !p_nid.contains(&id)  // non-cyclic
}

pub fn parent_id_of(nid: &Nid) -> Option<u32> {
    let len = nid.len();
    match len {
        ..=1 => None,
        _ => Some(nid[len - 2]),
    }
}

// description of a node in the tree structure.
// need to be generated by that node, passed to its parent and children.
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct NodeDesc {
    pub nid: Nid,  // structural id of node, down-flowing data
    pub p: PosVec,
    pub v: Velocity,
    pub swm: u32,  // the size of the swarm, down-flowing data
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct NodeDetails {
    pub subswarm: u32,  // the size of the subswarm, up-flowing data
    pub free: bool,  // whether the node is in free state, down-flowing data
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Line {
    pub points: Vec<PosVec>,
    pub closed: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Task {
    pub line: Line,
    pub duration: Duration,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum MsgBody {
    BROADCASTING,  // report basic status to all neighbours, no specific target
    CONNECTION(NodeDetails),  // repport status to parent and children
    JOIN(NodeDetails),
    LEAVE,
    TASK(Task),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Msg {
    pub sender: NodeDesc,  // node description of message sender
    pub to_ids: Vec<u32>,  // target message receivers, None means broadcasting
    pub body: MsgBody,
}