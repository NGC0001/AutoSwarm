use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};

use super::transceiver::Transceiver;

pub const CHANNEL: &str = "CTRL";

#[derive(Copy, Clone, Deserialize, Serialize, Debug)]
pub struct Velocity {
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
}

#[derive(Copy, Clone, Deserialize, Serialize, Debug)]
pub struct ControlMsg {
    pub v: Velocity,
}

pub struct Control {
    v: Velocity,
    tc: Rc<RefCell<Transceiver>>,
}

impl Control {
    pub fn new(tc: &Rc<RefCell<Transceiver>>) -> Control {
        Control {
            v: Velocity {vx: 0.0, vy: 0.0, vz: 0.0},
            tc: tc.clone(),
        }
    }

    pub fn read_v(&self) -> Velocity {
        self.v
    }

    pub fn set_v(&mut self, v: &Velocity) {
        self.v = *v;
        self.send_ctrl_msg();
    }

    pub fn send_ctrl_msg(&self) {
        let msg = ControlMsg {v: self.v};
        (*self.tc).borrow_mut().send(CHANNEL, &msg);
    }
}