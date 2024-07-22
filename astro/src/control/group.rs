use std::collections::{HashMap, HashSet};

use super::Position;

pub struct Member {
    grp_conn: HashSet<u32>,
    p: Position,
    parent_conn: HashSet<u32>,
    left_conn: HashSet<u32>,
    right_conn: HashSet<u32>,
}

impl Member {
    pub fn new(p: &Position) -> Member {
        Member {
            grp_conn: HashSet::new(),
            p: *p,
            parent_conn: HashSet::new(),
            left_conn: HashSet::new(),
            right_conn: HashSet::new(),
        }
    }
}

// the description of a direct child group or of the direct parent group.
// member of a group should be able to generate this information
// for its child groups and its parent group.
pub struct GrpConn {
    gid: (u32, u32),
    size: u32,
    centre: Position,
    subswarm_size: u32,
}

// group id: (founder id, tag)
// group level: gid(top) -> gid -> gid -> gid -> ... -> gid(this)
pub struct Group {
    level: Vec<(u32, u32)>,
    members: HashMap<u32, Member>,  // connection graph of the group
    subswarm_size: u32,
    parent: Option<GrpConn>,
    left: Option<GrpConn>,
    right: Option<GrpConn>,
}

impl Group {
    pub fn new_soliton(id: u32, p: &Position, tag: u32) -> Group {
        Group {
            level: vec![(id, tag)],
            members: HashMap::from([
                (id, Member::new(p)),
            ]),
            subswarm_size: 1,
            parent: None,
            left: None,
            right: None,
        }
    }
}