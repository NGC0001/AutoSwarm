use std::os::unix::net::UnixStream;
use std::{cell::RefCell, rc::Rc};
use std::time::{Duration, Instant};

use astro::control::Velocity;
use astro::gps::Position;
use astro::transceiver::Transceiver;

struct DataPack {
    p: Position,
    data_vec: Rc<Vec<String>>,
}

struct UavSim {
    id: u32,
    p: Position,
    p_calc_t: Instant,
    p_send_t: Instant,
    v: Velocity,
    out_msg: Vec<String>,  // messages from this UAV, to be sent to other ones
    in_msg: Vec<String>,  // messages from other UAVs, to be received by this one
    tc: Rc<RefCell<Transceiver>>,
}

impl UavSim {
    fn new(id: u32, p: Position, stream: UnixStream) -> UavSim {
        let now = Instant::now();
        UavSim {
            id,
            p,
            p_calc_t: now,
            p_send_t: now - Duration::from_secs(1000),
            v: Velocity {vx: 0.0, vy: 0.0, vz: 0.0},
            out_msg: vec![],
            in_msg: vec![],
            tc: Rc::new(RefCell::new(Transceiver::new(stream))),
        }
    }
}