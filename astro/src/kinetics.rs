use std::{cell::RefCell, rc::Rc};
use std::ops;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use quantity::VectorF32;

use super::transceiver::Transceiver;

pub const CHANNEL: &str = "KNTC";

// TODO: time is represented by capsulated type Duration, but distance is by f32

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

impl Velocity {
    pub fn limit_norm_to(&mut self, limit: f32) {
        let norm = self.norm();
        if norm > limit {
            *self *= limit / norm;
        }
    }

    pub fn get_norm_limited(&self, limit: f32) -> Velocity {
        let mut v = *self;
        v.limit_norm_to(limit);
        v
    }

    pub fn paral_component_to(&self, p: &PosVec) -> f32 {
        let direct = p.unit().unwrap();
        let product = direct.x * self.vx + direct.y * self.vy + direct.z * self.vz;  // inner product
        product
    }

    pub fn paral_to(&self, p: &PosVec) -> Velocity {
        let direct = p.unit().unwrap();
        let product = direct.x * self.vx + direct.y * self.vy + direct.z * self.vz;  // inner product
        Velocity {
            vx: direct.x * product,
            vy: direct.y * product,
            vz: direct.z * product,
        }
    }

    pub fn perp_to(&self, p: &PosVec) -> Velocity {
        self - self.paral_to(p)
    }
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

impl ops::Div<Duration> for &PosVec {
    type Output = Velocity;

    fn div(self, dt: Duration) -> Self::Output {
        let dt_s: f32 = dt.as_secs_f32();
        Velocity {
            vx: self.x / dt_s,
            vy: self.y / dt_s,
            vz: self.z / dt_s,
        }
    }
}

impl ops::Div<Duration> for PosVec {
    type Output = Velocity;

    fn div(self, dt: Duration) -> Self::Output {
        &self / dt
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
    max_v: f32,
}

impl Kinetics {
    pub fn new(max_v: f32, tc: &Rc<RefCell<Transceiver>>, v_init: &Velocity) -> Kinetics {
        Kinetics {
            v: *v_init,
            tc: tc.clone(),
            max_v,
        }
    }

    pub fn read_v(&self) -> Velocity {
        self.v
    }

    pub fn set_v(&mut self, v: &Velocity) {
        self.v = *v;
        self.v.limit_norm_to(self.max_v);  // ensure that the velocity doesn't exceed limit.
        self.send_kntc_msg();
    }

    pub fn send_kntc_msg(&self) {
        let msg = KntcMsg {v: self.v};
        (*self.tc).borrow_mut().send(CHANNEL, &msg);
    }
}