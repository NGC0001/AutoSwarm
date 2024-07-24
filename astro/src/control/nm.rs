use std::option::Option;

use crate::kinetics::{Position, Velocity};

use super::msg::{Nid, nid2id, parent_id_from_nid, NodeDesc, Msg};

pub struct NodeManager {
    nid: Nid,
    parent: Option<NodeDesc>,  // need backup ids (indirect upper nodes / sibling nodes)
    children: Vec<NodeDesc>,
    p: Position,
    v: Velocity,
}

impl NodeManager {
    pub fn new_root(id: u32, p: &Position, v: &Velocity) -> NodeManager {
        NodeManager {
            nid: vec![id],
            parent: None,
            children: vec![],
            p: *p,
            v: *v,
        }
    }

    pub fn calc_next_v(&self) -> Velocity {
        unimplemented!("next v");
        Velocity {
            vx: 0.0,
            vy: 0.0,
            vz: 0.0,
        }
    }

    pub fn generate_desc_msg(&self) -> Msg {
        let mut to_ids: Vec<u32> = vec![];
        if let Some(nd) = &self.parent {
            to_ids.push(nid2id(&nd.nid));
        }
        for nd in &self.children {
            to_ids.push(nid2id(&nd.nid));
        }
        let mut msg = Msg::new(self.generate_node_desc());
        msg.to_ids.append(&mut to_ids);
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
            v: self.v,
            subswarm_size,
            swarm_size,
        }
    }

    pub fn update_node(&mut self, p: &Position, v: &Velocity, rm: &Vec<u32>, msgs: &Vec<&Msg>) {
        self.p = *p;
        self.v = *v;
        if self.should_remove_parent(rm) {
            self.remove_parent();
        }
        self.children.retain(|ndesc| !rm.contains(&nid2id(&ndesc.nid)));
        for msg in msgs {
            self.update_desc(&msg.sender);
        }
    }

    fn should_remove_parent(&self, rm: &Vec<u32>) -> bool {
        match &self.parent {
            None => false,
            Some(pnd) => {  // parent node
                let pid = nid2id(&pnd.nid);  // parent id
                rm.contains(&pid)
            },
        }
    }

    fn remove_parent(&mut self) {
        let id: u32 = nid2id(&self.nid);
        self.parent = None;
        self.nid = vec![id];
    }

    fn update_desc(&mut self, desc: &NodeDesc) -> bool {
        self.update_parent_desc(desc) || self.update_child_desc(desc)
    }

    // TODO: when c recognise p as its parent, what if p does not recognise c as its child?
    fn update_parent_desc(&mut self, desc: &NodeDesc) -> bool {
        match &mut self.parent {
            None => false,
            Some(pnd) => {
                if nid2id(&pnd.nid) != nid2id(&desc.nid) {
                    false
                } else {  // the description is from recognised parent
                    let id = nid2id(&self.nid);
                    if desc.nid.contains(&id) {  // invalid parent, remove it
                        self.remove_parent();
                    } else {  // update parent description
                        *pnd = desc.clone();
                        self.nid = desc.nid.clone();
                        self.nid.push(id);
                    }
                    true
                }
            },
        }
    }

    fn update_child_desc(&mut self, desc: &NodeDesc) -> bool {
        let id = nid2id(&self.nid);
        for (idx, cnd) in self.children.iter_mut().enumerate() {
            if nid2id(&cnd.nid) == nid2id(&desc.nid) {  // the description is from a recognised child
                if parent_id_from_nid(&desc.nid).is_some_and(|v| v == id) {  // update description
                    *cnd = desc.clone();
                } else {  // the child does not recognised this node as its parent, remove it
                    self.children.remove(idx);
                }
                return true;
            }
        }
        false
    }
}