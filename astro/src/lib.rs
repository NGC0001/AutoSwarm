use std::os::unix::net::UnixStream;
use std::thread;
use std::time::{Duration, Instant};

pub mod comm;
pub mod gps;
pub mod kinetics;
pub mod transceiver;
pub mod util;use std::{cell::RefCell, rc::Rc};

use comm::{Comm, CommMsg};
use kinetics::{Kinetics, Velocity};
use gps::Gps;
use transceiver::Transceiver;

pub const EVENT_LOOP_INTERVAL_MIN: Duration = Duration::from_millis(70);
pub const EVENT_LOOP_INTERVAL: Duration = Duration::from_millis(100);

pub struct AstroConf {
    pub id: u32,  // starting from 1, 0 is reserved
    pub uav_radius: f32,
}

impl AstroConf {
    pub fn validate(&self) -> Result<(), ()> {
        if self.id == 0 {
            return Err(());
        }
        if self.uav_radius <= 0.0 {
            return Err(());
        }
        Ok(())
    }
}

pub struct Astro {
    conf: AstroConf,
    gps: Gps,
    kntc: Kinetics,
    comm: Comm,
}

impl Astro {
    pub fn new(conf: AstroConf) -> Astro {
        let socket_file: String = util::get_socket_name(conf.id);
        let stream = UnixStream::connect(socket_file).unwrap();
        let transceiver = Rc::new(RefCell::new(Transceiver::new(stream)));
        Astro {
            conf,
            gps: Gps::new(&transceiver),
            kntc: Kinetics::new(&transceiver),
            comm: Comm::new(&transceiver),
        }
    }

    pub fn init(&mut self) {
        loop {
            if self.gps.update() {
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }
    }

    pub fn run_event_loop(&mut self) {
        loop {
            let start = Instant::now();
            self.event_step();
            let end = Instant::now();
            if end - start < EVENT_LOOP_INTERVAL_MIN {
                let sleep_duration = EVENT_LOOP_INTERVAL - (end - start);
                thread::sleep(sleep_duration);
            }
        }
    }

    pub fn event_step(&mut self) {
        self.gps.update();
        self.kntc.set_v(&Velocity {vx: 0.0, vy: 0.0, vz: 0.0});
        let msg = CommMsg {
            from_id: self.conf.id,
            to_id: 0,
        };
        self.comm.send_msg(&msg);
        let msgs = self.comm.receive_msgs();
        if self.conf.id == 1 {
            dbg!(self.conf.id, self.gps.read_pos(), self.kntc.read_v(), &msgs);
        }
    }
}
