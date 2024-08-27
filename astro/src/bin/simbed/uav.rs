use std::option::Option;
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::process::{Child, Command};
use std::rc::Rc;

use astro::transceiver;

use super::uavconf::UavConf;
use super::uavsim::UavSim;

#[derive(PartialEq)]
enum UavStatus {
    Starting,
    Running,
    Shutdown,
}

pub struct Uav {
    conf: Rc<UavConf>,
    status: UavStatus,
    socket_file: String,
    listener: UnixListener,
    process: Child,
    sim: Option<UavSim>,
}

impl Uav {
    pub fn new(conf: UavConf, bin: &String) -> Uav {
        let conf = Rc::new(conf);
        let socket_file: String = transceiver::get_socket_name(conf.id);
        if Path::new(&socket_file).exists() {
            std::fs::remove_file(&socket_file).unwrap();
        }
        let listener = UnixListener::bind(socket_file.clone()).unwrap();
        listener.set_nonblocking(true).unwrap();
        let process = Self::spawn_uav_process(&*conf, bin);
        Uav {
            conf,
            status: UavStatus::Starting,
            socket_file,
            listener,
            process,
            sim: Option::None,
        }
    }

    fn spawn_uav_process(conf: &UavConf, bin: &String) -> Child {
        Command::new(bin)
            .arg("--id").arg(conf.id.to_string())
            .arg("--uav-radius").arg(conf.radius.to_string())
            .arg("--msg-range").arg(conf.msg_out_distance.to_string())
            .arg("--max-v").arg(conf.max_v.to_string())
            .spawn().unwrap()
    }

    pub fn shutdown(&mut self) {
        self.process.kill().unwrap();
        self.sim = Option::None;
        self.status = UavStatus::Shutdown;
    }

    pub fn is_shutdown(&self) -> bool {
        match &self.status {
            UavStatus::Shutdown => true,
            _ => false,
        }
    }

    pub fn get_uav_sim(&mut self) -> &mut Option<UavSim> {
        if self.status == UavStatus::Starting {
            self.try_accept();
        }
        &mut self.sim
    }

    fn try_accept(&mut self) -> bool {
        match self.listener.accept() {
            Ok((stream, _)) => {
                std::fs::remove_file(&self.socket_file).unwrap();
                self.sim = Option::Some(UavSim::new(&self.conf, stream));
                self.status = UavStatus::Running;
                true
            },
            Err(_) => false,
        }
    }
}