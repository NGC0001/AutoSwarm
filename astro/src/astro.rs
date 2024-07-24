use std::{cell::RefCell, rc::Rc};
use std::os::unix::net::UnixStream;
use std::thread;
use std::time::{Duration, Instant};

use crate::kinetics::{Position, Velocity};

use super::astroconf::AstroConf;
use super::comm::Comm;
use super::control::Control;
use super::kinetics::Kinetics;
use super::gps::Gps;
use super::transceiver::Transceiver;
use super::util;

pub const EVENT_LOOP_INTERVAL_MIN: Duration = Duration::from_millis(70);
pub const EVENT_LOOP_INTERVAL: Duration = Duration::from_millis(100);

pub struct Astro {
    gps: Gps,
    kntc: Kinetics,
    comm: Comm,
    ctrl: Control,
}

impl Astro {
    pub fn new(conf: AstroConf) -> Astro {
        let conf = Rc::new(conf);
        let socket_file: String = util::get_socket_name(conf.id);
        let stream = UnixStream::connect(socket_file).unwrap();
        let transceiver = Rc::new(RefCell::new(Transceiver::new(stream)));
        let p_dummy = Position {x: 0.0, y: 0.0, z: 0.0};
        let v_dummy = Velocity {vx: 0.0, vy: 0.0, vz: 0.0};
        Astro {
            gps: Gps::new(&transceiver, &p_dummy),
            kntc: Kinetics::new(&transceiver, &v_dummy),
            comm: Comm::new(&transceiver),
            ctrl: Control::new(&conf, &p_dummy, &v_dummy),
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
        let curr_v = self.kntc.read_v();
        let curr_p = self.gps.predict_pos(&curr_v);
        let msgs = self.comm.receive_msgs();
        let (next_v, msgs_out) = self.ctrl.update(&curr_p, &curr_v, &msgs);
        self.kntc.set_v(&next_v);
        self.comm.send_msgs(&msgs_out);
    }
}
