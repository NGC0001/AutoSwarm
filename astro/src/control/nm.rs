use std::option::Option;

use super::msg::{Nid, NodeDesc};

pub struct NodeManager {
    nid: Nid,
    parent: Option<NodeDesc>,
    children: Vec<NodeDesc>,
    backup: Vec<u32>,  // stores ids of indirect upper nodes or nodes in sibling subtrees.
}

impl NodeManager {
    pub fn new_root(id: u32) -> NodeManager {
        NodeManager {
            nid: vec![id],
            parent: None,
            children: vec![],
            backup: vec![],
        }
    }
}