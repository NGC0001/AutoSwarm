use std::{cell::RefCell, rc::Rc};

use super::transceiver::Transceiver;
use super::control::msg::Msg as CommMsg;

pub const CHANNEL: &str = "COMM";

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