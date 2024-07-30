// a very simple collision avoidance module

// TODO: test this module, maybe using unit tests

use std::rc::Rc;
use std::time::Duration;

use super::super::astroconf::AstroConf;
use super::super::kinetics::{distance, PosVec, Velocity};
use super::msg::NodeDesc;

pub const DEFAULT_TIME_SCALE: Duration = Duration::from_millis(1000);
pub const DEFAULT_MODEST_NUM_DANGERS: usize = 2;

pub struct ColliVoid {
    t_scale: Duration,
    modest_num_dangers: usize,
    evasion_dist: f32,
}

impl ColliVoid {
    pub fn new(conf: &Rc<AstroConf>) -> ColliVoid {
        ColliVoid {
            t_scale: DEFAULT_TIME_SCALE,  // depends on acceleration
            modest_num_dangers: DEFAULT_MODEST_NUM_DANGERS,
            evasion_dist: conf.uav_radius * 3.0,
        }
    }

    pub fn get_safe_v(&self, v_aim: &Velocity, p_self: &PosVec, neighbours: &Vec<&NodeDesc>) -> Velocity {
        let dangers = self.pick_dangers(v_aim, p_self, neighbours);
        let capped_v = self.get_capped_v(v_aim, p_self, neighbours);
        if dangers.is_empty() {
            return capped_v;
        }
        let nearest: &NodeDesc = &dangers[0];
        if distance(&nearest.p, p_self) > self.evasion_dist {
            return capped_v;
        }
        // TODO: just evading the nearest target is not enough,
        // use artificial potential field method to avoid hitting nearby targets
        self.evade(&capped_v, p_self, &nearest)
    }

    // TODO: maybe considering the velocity of `target`
    // TODO: detour rather than just stop
    fn evade(&self, v: &Velocity, p_self: &PosVec, target: &NodeDesc) -> Velocity {
        let direct: PosVec = (target.p - p_self).unit().unwrap();
        let component = direct.x * v.vx + direct.y * v.vy + direct.z * v.vz;  // inner product
        if component <= 0.0 {
            return *v;
        }
        v - component * Velocity {  // strip off the velocity component flying towards the target
            vx: direct.x,
            vy: direct.y,
            vz: direct.z,
        }
    }

    fn get_capped_v(&self, v_aim: &Velocity, p_self: &PosVec, dangers: &Vec<&NodeDesc>) -> Velocity {
        if dangers.len() <= self.modest_num_dangers {
            return *v_aim;
        }
        let mut v_ave: Velocity = Velocity::zero();
        for d in dangers {
            v_ave += d.v;
        }
        v_ave /= dangers.len() as f32;
        let v_delta_cap: f32 = ((dangers[self.modest_num_dangers].p - p_self) / self.t_scale).norm();
        let v_delta: Velocity = v_aim - v_ave;
        if v_delta.norm() <= v_delta_cap {
            *v_aim
        } else {
            v_ave + v_delta.unit().unwrap() * v_delta_cap
        }
    }

    // result is sorted by distance
    fn pick_dangers<'a>(&self, v_aim: &Velocity, p_self: &PosVec, neighbours: &Vec<&'a NodeDesc>)
    -> Vec<&'a NodeDesc> {
        let safe_d: f32 = (v_aim * self.t_scale).norm();
        let mut dangers: Vec<&NodeDesc> = neighbours.iter().map(|nd| *nd).filter(
            |nd| distance(&nd.p, p_self) <= safe_d).collect();
        dangers.sort_unstable_by(|nd1, nd2| {
            distance(&nd1.p, p_self).partial_cmp(&distance(&nd2.p, p_self)).unwrap()
        });
        dangers
    }
}