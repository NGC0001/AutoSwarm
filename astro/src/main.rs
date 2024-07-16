use std::{cell::RefCell, rc::Rc};
use std::os::unix::net::UnixStream;
use std::{thread, time};

use clap::Parser;

use astro::{comm::Comm, comm::CommMsg};
use astro::{control::Control, control::Velocity};
use astro::gps::Gps;
use astro::transceiver::Transceiver;
use astro::util;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t = 0)]
    id: u32,
}

fn main() {
    let args = Args::parse();
    let socket_file: String = util::get_socket_name(args.id);
    let stream = UnixStream::connect(socket_file).unwrap();
    let transceiver = Rc::new(RefCell::new(Transceiver::new(stream)));
    let mut gps = Gps::new(&transceiver);
    let mut control = Control::new(&transceiver);
    let comm = Comm::new(&transceiver);
    loop {
        if gps.update() {
            break;
        }
        thread::sleep(time::Duration::from_millis(100));
    }
    loop {
        gps.update();
        control.set_v(&Velocity {vx: 0.0, vy: 0.0, vz: 0.0});
        let msg = CommMsg {
            from_id: args.id,
            to_id: -1,
        };
        comm.send_msg(&msg);
        let msgs = comm.receive_msgs();
        dbg!(args.id, gps.read_pos(), control.read_v(), &msgs);
        thread::sleep(time::Duration::from_millis(100));
    }
}