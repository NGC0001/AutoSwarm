use std::thread;
use std::time::{Duration, Instant};

use rand::{thread_rng, seq::SliceRandom};

use astro::kinetics::PosVec;

use super::uavconf::UavConf;
use super::uavsim::{UavSim, MsgPack};
use super::uav::Uav;

pub const SIM_LOOP_INTERVAL_MIN: Duration = Duration::from_millis(30);
pub const SIM_LOOP_INTERVAL: Duration = Duration::from_millis(50);
pub const UAV_INIT_POS_INTERVAL: f32 = 5.0;  // m

// provide simulation support for a UAV swarm including:
// a) kinetic integration for each UAV
// b) message distribution among the swarm
// c) collision check
pub struct SimBed {
    uavs: Vec<Uav>,  // UAV with `id` should be placed at index `id - 1`
}

impl SimBed {
    pub fn new(num_uav: u32, astro_bin: &String) -> SimBed {
        let mut init_p_vec: Vec<PosVec> = vec![];
        let row_len = (num_uav as f64).sqrt().ceil() as u32;
        for id in 0..num_uav {
            let row: u32 = id / row_len;
            let col: u32 = id % row_len;
            init_p_vec.push(PosVec {
                x: UAV_INIT_POS_INTERVAL * (col as f32),
                y: UAV_INIT_POS_INTERVAL * (row as f32),
                z: 0.0,
            });
        }
        init_p_vec.shuffle(&mut thread_rng());
        let mut uavs: Vec<Uav> = vec![];
        for id in 0..num_uav {
            let conf = UavConf::new(id, init_p_vec[id as usize]);
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
            if self.uavs.iter().all(|uav| uav.is_shutdown()) {
                // all uavs have been shutdown
                break;
            }
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
        // println!("message packs: {:?}\n",
        //     msg_packs.iter().map(|p| (p.get_source_id(), p.get_num_data())).collect::<Vec<(u32, usize)>>());
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
