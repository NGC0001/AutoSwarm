use astro::gps::Position;

use super::uavsim::{self, UavConf, UavSim};
use super::uav::Uav;

pub struct SimBed {
    uavs: Vec<Uav>,
}

impl SimBed {
    pub fn new(num_uav: u32, astro_bin: &String) -> SimBed {
        let mut uavs: Vec<Uav> = vec![];
        for id in 0..num_uav {
            let init_p = Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            };
            let conf = UavConf {
                id,
                msg_out_distance: uavsim::DEFAULT_MSG_OUT_DISTANCE,
                init_p,
            };
            uavs.push(Uav::new(conf, astro_bin));
        }
        SimBed {
            uavs,
        }
    }
}