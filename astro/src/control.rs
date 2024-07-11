use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};

use super::transceiver::Transceiver;

const CHANNEL: &str = "CTRL";

#[derive(Copy, Clone, Deserialize, Serialize, Debug)]
pub struct Velocity {
    vx: f64,
    vy: f64,
    vz: f64,
}

#[derive(Copy, Clone, Deserialize, Serialize, Debug)]
pub struct ControlMsg {
    v: Velocity,
}

pub struct Control {
    v: Velocity,
    tc: Rc<RefCell<Transceiver>>,
}

impl Control {
    pub fn new(tc: &Rc<RefCell<Transceiver>>) -> Control {
        Control {
            v: Velocity{vx: 0.0, vy: 0.0, vz: 0.0},
            tc: tc.clone(),
        }
    }

    pub fn read_v(&self) -> Velocity {
        self.v
    }

    pub fn set_v(&mut self, v: &Velocity) {
        self.v = *v;
        let msg = ControlMsg {v: self.v};
        (*self.tc).borrow_mut().send(CHANNEL, &msg);
    }
}