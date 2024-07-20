use std::{cell::RefCell, rc::Rc};
use std::time::Instant;

use serde::{Deserialize, Serialize};

use super::kinetics::{Position, Velocity};
use super::transceiver::Transceiver;

pub const CHANNEL: &str = "GPS_";

#[derive(Deserialize, Serialize, Debug)]
pub struct GpsMsg {
    pub p: Position,
}

pub struct Gps {
    p: Position,
    p_predict: Position,
    p_predict_t: Instant,
    tc: Rc<RefCell<Transceiver>>,
}

impl Gps {
    pub fn new(tc: &Rc<RefCell<Transceiver>>) -> Gps {
        Gps {
            p: Position {x: 0.0, y: 0.0, z: 0.0},
            p_predict: Position {x: 0.0, y: 0.0, z: 0.0},
            p_predict_t: Instant::now(),
            tc: tc.clone(),
        }
    }

    pub fn read_pos(&self) -> Position {
        self.p
    }

    pub fn predict_pos(&mut self, v: &Velocity) -> Position {
        let now = Instant::now();
        self.p_predict += v * (now - self.p_predict_t);
        self.p_predict_t = now;
        self.p_predict
    }

    pub fn update(&mut self) -> bool {
        let msgs = (*self.tc).borrow_mut().retrieve::<GpsMsg>(CHANNEL);
        match msgs.last() {
            None => false,
            Some(m) => {
                self.p = m.p;
                self.p_predict = m.p;
                self.p_predict_t = Instant::now();
                true
            },
        }
    }
}