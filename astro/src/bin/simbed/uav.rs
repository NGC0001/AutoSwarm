use std::option::Option;
use std::os::unix::net::{UnixListener, UnixStream};

use astro::util;

use super::uavsim::UavSim;

pub struct Uav {
    id: u32,
    // https://doc.rust-lang.org/std/os/unix/net/struct.Incoming.html
    // for stream in listener.incoming()
    listener: UnixListener,
    sim: Option<UavSim>,
}

impl Uav {
    pub fn new(id: u32) -> Uav {
        let socket_file: String = util::get_socket_name(id);
        Uav {
            id,
            listener: UnixListener::bind(socket_file).unwrap(),
            sim: Option::None,
        }
    }
}