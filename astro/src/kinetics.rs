use std::{cell::RefCell, rc::Rc};
use std::ops;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::transceiver::Transceiver;

pub const CHANNEL: &str = "KNTC";

#[derive(Copy, Clone, Deserialize, Serialize, Debug)]
pub struct Position {  // m
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Copy, Clone, Deserialize, Serialize, Debug)]
pub struct Velocity {  // m/s
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
}

pub struct Displacement {  // m
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl ops::Mul<Duration> for &Velocity {
    type Output = Displacement;

    fn mul(self, dt: Duration) -> Self::Output {
        let dt_s: f32 = dt.as_secs_f32();
        Displacement {
            x: self.vx * dt_s,
            y: self.vy * dt_s,
            z: self.vz * dt_s,
        }
    }
}

impl ops::Mul<Duration> for Velocity {
    type Output = Displacement;

    fn mul(self, dt: Duration) -> Self::Output {
        &self * dt
    }
}

impl ops::Add<&Displacement> for &Position {
    type Output = Position;

    fn add(self, d: &Displacement) -> Self::Output {
        Position {
            x: self.x + d.x,
            y: self.y + d.y,
            z: self.z + d.z,
        }
    }
}

impl ops::Add<Displacement> for &Position {
    type Output = Position;

    fn add(self, d: Displacement) -> Self::Output {
        self + &d
    }
}

impl ops::AddAssign<&Displacement> for Position {
    fn add_assign(&mut self, d: &Displacement) {
        self.x += d.x;
        self.y += d.y;
        self.z += d.z;
    }
}

impl ops::AddAssign<Displacement> for Position {
    fn add_assign(&mut self, d: Displacement) {
        *self += &d;
    }
}

#[derive(Copy, Clone, Deserialize, Serialize, Debug)]
pub struct KntcMsg {
    pub v: Velocity,
}

pub struct Kinetics {
    v: Velocity,
    tc: Rc<RefCell<Transceiver>>,
}

impl Kinetics {
    pub fn new(tc: &Rc<RefCell<Transceiver>>) -> Kinetics {
        Kinetics {
            v: Velocity {vx: 0.0, vy: 0.0, vz: 0.0},
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