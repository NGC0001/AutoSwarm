use std::{cell::RefCell, rc::Rc};
use std::option::Option;

use serde::{Deserialize, Serialize};

use super::groupstate::Sid;
use super::kinetics::Position;
use super::transceiver::Transceiver;

pub const CHANNEL: &str = "COMM";

#[derive(Deserialize, Serialize, Debug)]
pub struct CommMsg {
    pub from_sid: Sid,  // hierarchical id of message sender
    pub from_p: Position,  // position of message sender
    pub to_ids: Option<Vec<u32>>,  // target message receivers, None means broadcasting
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