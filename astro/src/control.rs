use std::rc::Rc;

use crate::Astro;

use super::astroconf::AstroConf;
use super::kinetics::{Position, Velocity};
use super::comm::CommMsg;

pub struct Control {
    conf: Rc<AstroConf>,
}

impl Control {
    pub fn new(conf: &Rc<AstroConf>) -> Control {
        Control {
            conf: conf.clone(),
        }
    }

    pub fn process(&mut self, p: &Position, v: &Velocity, msgs_in: &Vec<CommMsg>)
    -> (Velocity, Vec<CommMsg>) {
        let next_v = Velocity {vx: 0.0, vy: 0.0, vz: 0.0};
        let mut msgs_out: Vec<CommMsg> = vec![];
        (next_v, msgs_out)
    }
}