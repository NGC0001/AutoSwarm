use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};

use super::transceiver::Transceiver;

const CHANNEL: &str = "GPS_";

#[derive(Copy, Clone, Deserialize, Serialize, Debug)]
pub struct Position {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Copy, Clone, Deserialize, Serialize, Debug)]
pub struct GpsMsg {
    p: Position,
}

pub struct Gps {
    p: Position,
    tc: Rc<RefCell<Transceiver>>,
}

impl Gps {
    pub fn new(tc: &Rc<RefCell<Transceiver>>) -> Gps {
        Gps {
            p: Position {x: 0.0, y: 0.0, z: 0.0},
            tc: tc.clone(),
        }
    }

    pub fn read_pos(&self) -> Position {
        self.p
    }

    pub fn update(&mut self) {
        for m in (*self.tc).borrow_mut().retrieve::<GpsMsg>(CHANNEL) {
            self.p = m.p;
        }
    }
}