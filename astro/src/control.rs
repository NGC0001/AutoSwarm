use std::{cell::RefCell, rc::Rc};
use super::transceiver::Transceiver;

pub struct Velocity {
    vx: i32,
    vy: i32,
    vz: i32,
}

pub struct Control {
    v: Velocity,
    tc: Rc<RefCell<Transceiver>>,
}

impl Control {
    pub fn set_v(&mut self, vx: i32, vy: i32, vz: i32) {
        self.v.vx = vx;
        self.v.vy = vy;
        self.v.vz = vz;
    }

    pub fn new(tc: &Rc<RefCell<Transceiver>>) -> Control {
        let c = Control {v: Velocity{vx: 0, vy: 0, vz: 0}, tc: tc.clone()};
        c
    }
}