use std::{cell::RefCell, rc::Rc};
use std::ops;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use quantity::VectorF32;

use super::transceiver::Transceiver;

pub const CHANNEL: &str = "KNTC";

// the position vector (i.e., displacement)
#[derive(VectorF32, Copy, Clone, Deserialize, Serialize, Debug)]
pub struct PosVec {  // m
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(VectorF32, Copy, Clone, Deserialize, Serialize, Debug)]
pub struct Velocity {  // m/s
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
}

impl ops::Mul<Duration> for &Velocity {
    type Output = PosVec;

    fn mul(self, dt: Duration) -> Self::Output {
        let dt_s: f32 = dt.as_secs_f32();
        PosVec {
            x: self.vx * dt_s,
            y: self.vy * dt_s,
            z: self.vz * dt_s,
        }
    }
}

impl ops::Mul<Duration> for Velocity {
    type Output = PosVec;

    fn mul(self, dt: Duration) -> Self::Output {
        &self * dt
    }
}

pub fn distance(p1: &PosVec, p2: &PosVec) -> f32 {
    (p1 - p2).norm()
}

#[derive(Deserialize, Serialize, Debug)]
pub struct KntcMsg {
    pub v: Velocity,
}

pub struct Kinetics {
    v: Velocity,
    tc: Rc<RefCell<Transceiver>>,
}

impl Kinetics {
    pub fn new(tc: &Rc<RefCell<Transceiver>>, v_init: &Velocity) -> Kinetics {
        Kinetics {
            v: *v_init,
            tc: tc.clone(),
        }
    }

    pub fn read_v(&self) -> Velocity {
        self.v
    }

    pub fn set_v(&mut self, v: &Velocity) {
        self.v = *v;
        self.send_kntc_msg();
    }

    pub fn send_kntc_msg(&self) {
        let msg = KntcMsg {v: self.v};
        (*self.tc).borrow_mut().send(CHANNEL, &msg);
    }
}