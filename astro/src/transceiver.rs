use std::any::Any;
use std::collections::HashMap;
use std::io::{BufReader, BufWriter};
use std::os::unix::net::UnixStream;
use std::path::Path;

pub struct Transceiver {
    stream: UnixStream,
    msg_map: HashMap<&'static str, Vec<Box<dyn Any>>>,
}

impl Transceiver {
    pub fn new(path: &String) -> Transceiver {
        let socket_path = Path::new(path);
        let stream: UnixStream = UnixStream::connect(socket_path).unwrap();
        Transceiver {stream: stream, msg_map: HashMap::new()}
    }

    pub fn retrieve(&mut self, channel: &str) -> Vec<Box<dyn Any>> {
        let mut res: Vec<Box<dyn Any>> = vec![];
        if let Some(messages) = self.msg_map.get_mut(channel) {
            std::mem::swap(messages, &mut res);
        }
        res
    }
}