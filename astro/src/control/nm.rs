use std::option::Option;

use crate::kinetics::Position;

use super::msg::{Nid, NodeDesc, Msg};

pub struct NodeManager {
    nid: Nid,
    parent: Option<NodeDesc>,  // need backup ids (indirect upper nodes / sibling nodes)
    children: Vec<NodeDesc>,
    p: Position,
}

impl NodeManager {
    pub fn new_root(id: u32, p: &Position) -> NodeManager {
        NodeManager {
            nid: vec![id],
            parent: None,
            children: vec![],
            p: *p,
        }
    }

    pub fn generate_desc_msg(&self) -> Msg {
        let mut to_ids: Vec<u32> = vec![];
        if let Some(nd) = &self.parent {
            to_ids.push(*nd.nid.last().unwrap());
        }
        for nd in &self.children {
            to_ids.push(*nd.nid.last().unwrap());
        }
        let mut msg = Msg::new(&self.nid, &self.p);
        msg.to_ids.append(&mut to_ids);
        msg.node_desc = Some(self.generate_node_desc());
        msg
    }

    pub fn generate_node_desc(&self) -> NodeDesc {
        let mut subswarm_size: u32 = 1;
        for nd in &self.children {
            subswarm_size += nd.subswarm_size;
        }
        let swarm_size: u32 = match &self.parent {
            None => subswarm_size,
            Some(nd) => nd.swarm_size,
        };
        NodeDesc {
            nid: self.nid.clone(),
            p: self.p,
            subswarm_size,
            swarm_size,
        }
    }

    pub fn update_node_conn(&mut self, p: &Position, rm: &Vec<u32>) {
        self.p = *p;
        if self.should_remove_parent(rm) {
            self.remove_parent();
        }
        self.children.retain(|ndesc| !rm.contains(ndesc.nid.last().unwrap()));
    }

    fn should_remove_parent(&self, rm: &Vec<u32>) -> bool {
        match &self.parent {
            None => false,
            Some(pnd) => {  // parent node
                let pid = pnd.nid.last().unwrap();  // parent id
                rm.contains(pid)
            },
        }
    }

    fn remove_parent(&mut self) {
        let id: u32 = *self.nid.last().unwrap();
        self.parent = None;
        self.nid = vec![id];
    }
}