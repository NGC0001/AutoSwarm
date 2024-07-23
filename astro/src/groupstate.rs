use std::collections::{HashMap, HashSet};

use super::kinetics::Position;

// group id: (founder id, tag)
pub type GrpId = (u32, u32);
// group level: gid(top) -> gid -> gid -> gid -> ... -> gid(this)
pub type GrpLevel = Vec<GrpId>;
// structural id of uav: (uav id, group level)
pub type Sid = (u32, GrpLevel);

pub struct Member {
    p: Position,
    grp_conn: HashSet<u32>,
    parent_conn: HashSet<u32>,
    left_conn: HashSet<u32>,
    right_conn: HashSet<u32>,
    version: u64,  // indicate the freshness of member connection data.
                   // as the data is generated from a single source (i.e., this member),
                   // the indicator can assure consistency across a group.
}

impl Member {
    pub fn new(p: &Position) -> Member {
        Member {
            p: *p,
            grp_conn: HashSet::new(),
            parent_conn: HashSet::new(),
            left_conn: HashSet::new(),
            right_conn: HashSet::new(),
            version: 0,
        }
    }
}

// the description of a group.
// member of a group should be able to generate this information,
// so as to send to its child groups and its parent group.
pub struct GrpDesc {
    gid: GrpId,
    size: u32,
    centre: Position,
    subswarm_size: u32,  // up-flowing data
    swarm_size: u32,  // down-flowing data
}

pub struct GrpState {
    level: GrpLevel,
    members: HashMap<u32, Member>,  // connection graph of the group
    parent: Option<GrpDesc>,
    children: Vec<GrpDesc>,
}

impl GrpState {
    pub fn new_soliton(id: u32, p: &Position, tag: u32) -> GrpState {
        GrpState {
            level: vec![(id, tag)],
            members: HashMap::from([
                (id, Member::new(p)),
            ]),
            parent: None,
            children: vec![],
        }
    }
}

pub struct Bill {
    bid: u64,  // should be bytes combination of (issuer_id: u32, tag: u32)
    pros: HashSet<u32>,
    cons: HashSet<u32>,
}