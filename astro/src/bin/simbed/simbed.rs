use astro::gps::Position;

use super::uavsim::{self, UavConf, UavSim, MsgPack};
use super::uav::Uav;

pub struct SimBed {
    uavs: Vec<Uav>,
}

impl SimBed {
    pub fn new(num_uav: u32, astro_bin: &String) -> SimBed {
        let mut uavs: Vec<Uav> = vec![];
        for id in 0..num_uav {
            let init_p = Position {
                x: 0.0 + (id as f64) * uavsim::DEFAULT_MSG_OUT_DISTANCE / 2.0,
                y: 0.0,
                z: 0.0,
            };
            let conf = UavConf {
                id,
                msg_out_distance: uavsim::DEFAULT_MSG_OUT_DISTANCE,
                init_p,
                p_send_intrvl: uavsim::DEFAULT_POSITION_SEND_INTERVAL,
            };
            uavs.push(Uav::new(conf, astro_bin));
        }
        SimBed {
            uavs,
        }
    }

    pub fn run_sim_loop(&mut self) {
        loop {
            self.sim_step();
        }
    }

    pub fn sim_step(&mut self) {
        let mut uav_sims: Vec<&mut UavSim> = vec![];
        for uav in &mut self.uavs {
            if let Some(uav_sim) = uav.get_uav_sim() {
                uav_sims.push(uav_sim);
            }
        }
        Self::update_kinetics(&mut uav_sims);
        let msg_packs = Self::collect_message_packs(&uav_sims);
        Self::dispose_message_packs(&uav_sims, &msg_packs);
    }

    fn update_kinetics(sims: &mut Vec<&mut UavSim>) {
        for sim in sims {
            sim.update_p();
            sim.update_v();
        }
    }

    fn collect_message_packs(sims: &Vec<&mut UavSim>) -> Vec<MsgPack> {
        let mut packs: Vec<MsgPack> = vec![];
        for sim in sims {
            packs.push(sim.collect_comm_msgs());
        }
        packs
    }

    fn dispose_message_packs(sims: &Vec<&mut UavSim>, msg_packs: &Vec<MsgPack>) {
        for sim in sims {
            sim.dispose_comm_msgs(msg_packs);
        }
    }
}