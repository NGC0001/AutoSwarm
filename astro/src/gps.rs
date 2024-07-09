use std::{cell::RefCell, rc::Rc};
use super::transceiver::Transceiver;

const CHANNEL: &str = "GPS";

#[derive(Copy, Clone)]
pub struct Position {
    x: f64,
    y: f64,
    z: f64,
}

pub struct Gps {
    p: Position,
    tc: Rc<RefCell<Transceiver>>,
}

impl Gps {
    pub fn get_pos(&self) -> &Position {
        &self.p
    }

    pub fn new(tc: &Rc<RefCell<Transceiver>>) -> Gps {
        let mut gps = Gps {p: Position{x: 0.0, y: 0.0, z: 0.0}, tc: tc.clone()};
        gps.update();
        gps
    }

    pub fn update(&mut self) {
        for msg in (*self.tc).borrow_mut().retrieve(CHANNEL) {
            if let Some(p) = msg.downcast_ref::<Position>() {
                self.p = *p;
            } else {
                panic!("Any not a Position");
            }
        }
    }
}