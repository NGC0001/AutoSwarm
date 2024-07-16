use std::option::Option;
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::process::{Child, Command};
use std::rc::Rc;

use astro::util;

use super::uavsim::{UavConf, UavSim};

pub struct Uav {
    conf: Rc<UavConf>,
    socket_file: String,
    listener: UnixListener,
    process: Child,
    sim: Option<UavSim>,
}

impl Uav {
    pub fn new(conf: UavConf, bin: &String) -> Uav {
        let conf = Rc::new(conf);
        let socket_file: String = util::get_socket_name(conf.id);
        if Path::new(&socket_file).exists() {
            std::fs::remove_file(&socket_file).unwrap();
        }
        let listener = UnixListener::bind(socket_file.clone()).unwrap();
        listener.set_nonblocking(true).unwrap();
        let process = Command::new(bin).arg("--id").arg(conf.id.to_string()).spawn().unwrap();
        Uav {
            conf,
            socket_file,
            listener,
            process,
            sim: Option::None,
        }
    }

    pub fn get_uav_sim(&mut self) -> &mut Option<UavSim> {
        if self.sim.is_none() {
            self.try_accept();
        }
        &mut self.sim
    }

    fn try_accept(&mut self) -> bool {
        match self.listener.accept() {
            Ok((stream, _)) => {
                self.sim = Option::Some(UavSim::new(&self.conf, stream));
                std::fs::remove_file(&self.socket_file).unwrap();
                true
            },
            Err(_) => false,
        }
    }
}