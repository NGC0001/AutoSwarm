use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};

use super::transceiver::Transceiver;

pub const CHANNEL: &str = "COMM";

#[derive(Copy, Clone, Deserialize, Serialize, Debug)]
pub struct CommMsg {
    pub from_id: u32,  // message sender
    pub to_id: u32,  // target message receiver, 0 means broadcasting
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

    pub fn send_msgs(&self, msgs: &Vec<CommMsg>) {
        let mut sender = (*self.tc).borrow_mut();
        for msg in msgs {
            sender.send(CHANNEL, msg);
        }
    }
}