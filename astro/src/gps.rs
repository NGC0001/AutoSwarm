use std::{cell::RefCell, rc::Rc};
use std::time::Instant;

use serde::{Deserialize, Serialize};

use super::kinetics::{PosVec, Velocity};
use super::transceiver::Transceiver;

pub const CHANNEL: &str = "GPS_";

#[derive(Deserialize, Serialize, Debug)]
pub struct GpsMsg {
    pub p: PosVec,
}

pub struct Gps {
    p: PosVec,
    p_predict: PosVec,
    p_predict_t: Instant,
    tc: Rc<RefCell<Transceiver>>,
}

impl Gps {
    pub fn new(tc: &Rc<RefCell<Transceiver>>, p_init: &PosVec) -> Gps {
        Gps {
            p: *p_init,
            p_predict: *p_init,
            p_predict_t: Instant::now(),
            tc: tc.clone(),
        }
    }

    pub fn read_pos(&self) -> PosVec {
        self.p
    }

    pub fn predict_pos(&mut self, v: &Velocity) -> PosVec {
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