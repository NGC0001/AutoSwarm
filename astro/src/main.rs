use std::{cell::RefCell, rc::Rc};
use std::os::unix::net::UnixStream;
use std::{thread, time};

use clap::Parser;

mod control;
mod gps;
mod transceiver;

use control::Velocity;
use transceiver::Transceiver;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t = 0)]
    id: u32,
}

fn main() {
    let args = Args::parse();
    let socket_file: String = format!("socket_{:06}", &args.id);
    dbg!(&args, &socket_file);
    let stream = UnixStream::connect(socket_file).unwrap();
    let transceiver = Rc::new(RefCell::new(Transceiver::new(stream)));
    let mut gps = gps::Gps::new(&transceiver);
    let mut control = control::Control::new(&transceiver);
    for _ in 0..5 {
        gps.update();
        dbg!(gps.read_pos());
        control.set_v(&Velocity {vx: 0.0, vy: 0.0, vz: 0.0});
        dbg!(control.read_v());
        thread::sleep(time::Duration::from_millis(1000));
    }
}
