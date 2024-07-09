use std::{cell::RefCell, rc::Rc};
use super::transceiver::Transceiver;

const CHANNEL: &str = "GPS";

pub struct Position {
    x: i32,
    y: i32,
    z: i32,
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
        let gps = Gps {p: Position{x: 0, y: 0, z: 0}, tc: tc.clone()};
        gps.update();
        gps
    }

    pub fn update(&self) {
        for msg in (*self.tc).borrow_mut().retrieve(CHANNEL) {}
    }
}