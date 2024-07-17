use std::thread;
use std::time::{Duration, Instant};

use astro::gps::Position;

use super::uavconf::{self, UavConf};
use super::uavsim::{UavSim, MsgPack};
use super::uav::Uav;

pub const SIM_LOOP_INTERVAL_MIN: Duration = Duration::from_millis(30);
pub const SIM_LOOP_INTERVAL: Duration = Duration::from_millis(50);

// provide simulation support for a UAV swarm including:
// a) kinetic integration for each UAV
// b) message distribution among the swarm
// c) collision check
pub struct SimBed {
    uavs: Vec<Uav>,  // UAV with `id` should be placed at index `id - 1`
}

impl SimBed {
    pub fn new(num_uav: u32, astro_bin: &String) -> SimBed {
        let mut uavs: Vec<Uav> = vec![];
        for id in 1..=num_uav {
            let init_p = Position {
                x: 0.0 + (id as f32) * uavconf::DEFAULT_MSG_OUT_DISTANCE / 1.8,
                y: 0.0,
                z: 0.0,
            };
            let conf = UavConf::new(id, init_p);
            uavs.push(Uav::new(conf, astro_bin));
        }
        SimBed {
            uavs,
        }
    }

    pub fn run_sim_loop(&mut self) {
        loop {
            let start = Instant::now();
            self.sim_step();
            let end = Instant::now();
            if end - start < SIM_LOOP_INTERVAL_MIN {
                let sleep_duration = SIM_LOOP_INTERVAL - (end - start);
                thread::sleep(sleep_duration);
            }
        }
    }

    // TODO: ready for multi-threading acceleration
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
        let collision_ids = Self::check_collisions_by_msg_packs(&uav_sims, &msg_packs);
        self.shutdown_uavs(collision_ids);
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

    fn check_collisions_by_msg_packs(sims: &Vec<&mut UavSim>, msg_packs: &Vec<MsgPack>) -> Vec<u32> {
        let mut collision_ids: Vec<u32> = vec![];
        for sim in sims {
            for pack in msg_packs {
                if sim.get_id() == pack.get_source_id() {  // no collision check with itself
                    continue;
                }
                if sim.overlap_with_uav_at(pack.get_source_p()) {
                    collision_ids.push(pack.get_source_id());
                    collision_ids.push(sim.get_id());
                }
            }
        }
        collision_ids
    }

    fn shutdown_uavs(&mut self, ids: Vec<u32>) {
        for id in ids {
            self.uavs[(id - 1) as usize].shutdown();
        }
    }
}
