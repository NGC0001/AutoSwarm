use std::option::Option;

use serde::{Deserialize, Serialize};

use super::Position;

// the node id of a uav in a tree structure.
// it is: id(top) -> id -> ... -> id(this)
pub type Nid = Vec<u32>;

#[derive(Deserialize, Serialize, Debug)]
pub struct NodeDesc {
    pub id: u32,
    pub p: Position,
    pub subswarm_size: u32,  // up-flowing data
    pub swarm_size: u32,  // down-flowing data
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Msg {
    pub from_nid: Nid,  // structural id of message sender
    pub from_p: Position,  // position of message sender
    pub to_ids: Option<Vec<u32>>,  // target message receivers, None means broadcasting
}