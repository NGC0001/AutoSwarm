use std::collections::{HashMap, HashSet};

use super::Position;
use super::{GrpId, GrpLevel, Sid, Member, GrpDesc, GrpState};

pub struct Group {
    state: GrpState,
}

impl Group {
    pub fn new_soliton(id: u32, p: &Position, tag: u32) -> Group {
        Group {
            state: GrpState::new_soliton(id, p, tag),
        }
    }

    pub fn modify_conn_for(&mut self, id: u32, add: &Vec<&Sid>, rm: &Vec<u32>) {
        unimplemented!("");
    }
}