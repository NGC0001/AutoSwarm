use std::{cell::RefCell, rc::Rc};
use super::transceiver::Transceiver;

#[derive(Copy, Clone)]
pub struct Velocity {
    vx: f64,
    vy: f64,
    vz: f64,
}

pub struct Control {
    v: Velocity,
    tc: Rc<RefCell<Transceiver>>,
}

impl Control {
    pub fn set_v(&mut self, v: &Velocity) {
        self.v = *v;
    }

    pub fn new(tc: &Rc<RefCell<Transceiver>>) -> Control {
        let c = Control {v: Velocity{vx: 0.0, vy: 0.0, vz: 0.0}, tc: tc.clone()};
        c
    }
}