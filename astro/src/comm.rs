use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};

use super::transceiver::Transceiver;

pub const CHANNEL: &str = "COMM";

#[derive(Copy, Clone, Deserialize, Serialize, Debug)]
pub struct CommMsg {
    pub from_id: u32,
    pub to_id: i32,
}

pub struct Comm {
    tc: Rc<RefCell<Transceiver>>,
}

impl Comm {
    pub fn new(tc: &Rc<RefCell<Transceiver>>) -> Comm {
        Comm {
            tc: tc.clone(),
        }
    }

    pub fn receive_msgs(&self) -> Vec<CommMsg> {
        (*self.tc).borrow_mut().retrieve::<CommMsg>(CHANNEL)
    }

    pub fn send_msg(&self, msg: &CommMsg) {
        (*self.tc).borrow_mut().send(CHANNEL, msg);
    }
}