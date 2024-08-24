use std::borrow::Borrow;
use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Write};
use std::thread;
use std::time::{Duration, Instant};

use chrono::Local;
use rand::{thread_rng, seq::SliceRandom};
use serde::Serialize;

use astro::kinetics::PosVec;
use astro::control::msg::Msg;

use crate::uavsim::UavInfo;

use super::gcs::Gcs;
use super::uav::Uav;
use super::uavconf::UavConf;
use super::uavsim::{UavSim, MsgPack};

pub const SIM_LOOP_INTERVAL_MIN: Duration = Duration::from_millis(30);
pub const SIM_LOOP_INTERVAL: Duration = Duration::from_millis(50);
pub const UAV_INIT_POS_INTERVAL: f32 = 5.0;  // m
pub const DEFAULT_DATA_DIRECTOR: &str = "data";
pub const DEFAULT_OUTPUT_DURATION: Duration = Duration::from_secs(2);

// used to record swarm status in a file
#[derive(Clone, Serialize, Debug)]
pub struct SwarmInfo {
    running_duration: Duration,  // duration since simulation start
    uavs: Vec<UavInfo>,
}

// provide simulation support for a UAV swarm including:
// a) kinetic integration for each UAV
// b) message distribution among the swarm
// c) collision check
pub struct SimBed {
    sim_start_t: Instant,
    uavs: Vec<Uav>,  // UAV with `id` should be placed at index `id - 1`
    gcs: Gcs,  // ground control station
    writer: BufWriter<File>,
    last_output_t: Instant,
}

impl SimBed {
    pub fn new(num_uav: u32, astro_bin: &String, task_book: &String) -> SimBed {
        let init_p_vec = Self::generate_initial_positions(num_uav);
        let mut uavs: Vec<Uav> = vec![];
        for id in 0..num_uav {
            let conf = UavConf::new(id, init_p_vec[id as usize]);
            uavs.push(Uav::new(conf, astro_bin));
        }
        create_dir_all(DEFAULT_DATA_DIRECTOR).expect("unable to create data directory");
        let fname = String::from(DEFAULT_DATA_DIRECTOR)
            + "/out-" + Local::now().format("%Y%m%d-%H%M%S").to_string().borrow();
        let f = File::create(fname).expect("unable to create output file");
        let writer = BufWriter::new(f);
        let now = Instant::now();
        SimBed {
            sim_start_t: now,
            uavs,
            gcs: Gcs::new(task_book),
            writer,
            last_output_t: now - DEFAULT_OUTPUT_DURATION,
        }
    }

    fn generate_initial_positions(num: u32) -> Vec<PosVec> {
        let mut init_p_vec: Vec<PosVec> = vec![];
        let row_len = (num as f64).sqrt().ceil() as u32;
        for i in 0..num {
            let row: u32 = i / row_len;
            let col: u32 = i % row_len;
            init_p_vec.push(PosVec {
                x: UAV_INIT_POS_INTERVAL * (col as f32),
                y: UAV_INIT_POS_INTERVAL * (row as f32),
                z: 0.0,
            });
        }
        init_p_vec.shuffle(&mut thread_rng());
        init_p_vec
    }

    pub fn run_sim_loop(&mut self) {
        loop {
            let start = Instant::now();
            self.sim_step(start);
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

    // ready for multi-threading acceleration
    pub fn sim_step(&mut self, now: Instant) {
        let running_duration = now - self.sim_start_t;
        let mut uav_sims: Vec<&mut UavSim> = vec![];
        for uav in &mut self.uavs {
            if let Some(uav_sim) = uav.get_uav_sim() {
                uav_sims.push(uav_sim);
            }
        }
        Self::update_kinetics(&mut uav_sims);
        let msg_packs = Self::collect_message_packs_and_update_nids(&mut uav_sims);
        Self::dispose_message_packs(&uav_sims, &msg_packs);
        Self::dispose_gcs_messages(&uav_sims, &self.gcs.generate_gcs_msgs(running_duration));

        if now - self.last_output_t > DEFAULT_OUTPUT_DURATION {
            Self::output_swarm_info(&uav_sims, &mut self.writer, running_duration);
            self.last_output_t = now;
        }

        let collision_ids = Self::check_collisions_by_msg_packs(&uav_sims, &msg_packs);
        self.shutdown_uavs(collision_ids);
    }

    fn update_kinetics(sims: &mut Vec<&mut UavSim>) {
        for sim in sims {
            sim.update_p();
            sim.update_v();
        }
    }

    fn collect_message_packs_and_update_nids(sims: &mut Vec<&mut UavSim>) -> Vec<MsgPack> {
        let mut packs: Vec<MsgPack> = vec![];
        for sim in sims {
            packs.push(sim.collect_comm_msgs_and_update_nid());
        }
        packs
    }

    fn dispose_message_packs(sims: &Vec<&mut UavSim>, msg_packs: &Vec<MsgPack>) {
        for sim in sims {
            sim.dispose_comm_msg_packs(msg_packs);
        }
    }

    fn dispose_gcs_messages(sims: &Vec<&mut UavSim>, gcs_msgs: &Vec<Msg>) {
        for sim in sims {
            sim.dispose_comm_msgs(gcs_msgs);
        }
    }

    fn output_swarm_info(sims: &Vec<&mut UavSim>, writer: &mut BufWriter<File>, running_duration: Duration) {
        let mut uavs: Vec<UavInfo> = vec![];
        for sim in sims {
            uavs.push(sim.get_info());
        }
        let swarm_info = SwarmInfo {
            running_duration,
            uavs,
        };
        let data = serde_json::to_string(&swarm_info).unwrap();
        writer.write_all(data.as_bytes()).unwrap();
        writer.write_all("\n".as_bytes()).unwrap();
        writer.flush().unwrap();
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
