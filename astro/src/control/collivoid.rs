// a very simple collision avoidance module

// TODO: test this module, maybe using unit tests

use std::rc::Rc;
use std::time::Duration;

use super::super::astroconf::AstroConf;
use super::super::kinetics::{distance, PosVec, Velocity};
use super::msg::NodeDesc;

pub const DEFAULT_TIME_SCALE: Duration = Duration::from_millis(2000);
pub const DEFAULT_MINIMAL_ALERT_DISTANCE_RATIO: f32 = 10.0;
pub const DEFAULT_MODEST_NUM_DANGERS: usize = 2;
pub const DEFAULT_EVASION_TIME_SCALE: Duration = Duration::from_millis(2000);
pub const DEFAULT_EVASION_DIST_RATIO: f32 = 3.0;

pub struct ColliVoid {
    t_scale: Duration,
    modest_num_dangers: usize,
    minimal_alert_dist: f32,
    evasion_dist: f32,
}

impl ColliVoid {
    pub fn new(conf: &Rc<AstroConf>) -> ColliVoid {
        ColliVoid {
            t_scale: DEFAULT_TIME_SCALE,  // depends on acceleration
            modest_num_dangers: DEFAULT_MODEST_NUM_DANGERS,
            minimal_alert_dist: conf.uav_radius * DEFAULT_MINIMAL_ALERT_DISTANCE_RATIO,
            evasion_dist: conf.uav_radius * DEFAULT_EVASION_DIST_RATIO,
        }
    }

    pub fn get_safe_v(&self, v_aim: &Velocity, p_self: &PosVec, neighbours: &Vec<&NodeDesc>) -> Velocity {
        let dangers = self.pick_dangers(v_aim, p_self, neighbours);
        if dangers.is_empty() {
            return *v_aim;
        }
        let capped_v = self.get_capped_v(v_aim, p_self, neighbours);
        let mut evasion_v_sum = capped_v;
        let mut weight_sum: f32 = 1.0;
        for (idx, d) in dangers.iter().take(self.modest_num_dangers).enumerate() {
            let direct = &d.p - p_self;
            if capped_v.paral_component_to(&direct) <= 0.0 {
                continue;
            }
            let evasion_v= capped_v.perp_to(&direct)
                + capped_v.paral_to(&direct).get_norm_limited((direct / DEFAULT_EVASION_TIME_SCALE).norm());
            let weight = 0.3_f32.powi(idx as i32);
            evasion_v_sum += evasion_v * weight;
            weight_sum += weight;
        }
        let evasion_v = evasion_v_sum / weight_sum;  // soft evasion
        self.evade(evasion_v, p_self, dangers[0])  // strict evasion if too close
    }

    // TODO: maybe considering the velocity of `danger`, and detour `danger`
    fn evade(&self, v: Velocity, p_self: &PosVec, danger: &NodeDesc) -> Velocity {
        if distance(&danger.p, p_self) > self.evasion_dist {
            return v;
        }
        let direct = &danger.p - p_self;
        if v.paral_component_to(&direct) <= 0.0 {
            return v;
        }
        v.perp_to(&direct)  // strip off the velocity component flying towards the danger
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
        let mut v_delta: Velocity = v_aim - v_ave;
        v_delta.limit_norm_to(v_delta_cap);
        v_ave + v_delta
    }

    // result is sorted by distance
    fn pick_dangers<'a>(&self, v_aim: &Velocity, p_self: &PosVec, neighbours: &Vec<&'a NodeDesc>)
    -> Vec<&'a NodeDesc> {
        let alert_d: f32 = f32::max((v_aim * self.t_scale).norm(), self.minimal_alert_dist);
        let mut dangers: Vec<&NodeDesc> = neighbours.iter().map(|nd| *nd).filter(
            |nd| distance(&nd.p, p_self) <= alert_d).collect();
        dangers.sort_unstable_by(|nd1, nd2| {
            distance(&nd1.p, p_self).partial_cmp(&distance(&nd2.p, p_self)).unwrap()
        });
        dangers
    }
}