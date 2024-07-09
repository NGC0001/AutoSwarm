use std::collections::HashMap;
use std::io::{BufReader, BufWriter};
use std::os::unix::net::UnixStream;
use std::path::Path;

pub struct Transceiver {
    stream: UnixStream,
    msg_map: HashMap<&'static str, Vec<String>>,
}

impl Transceiver {
    pub fn new(path: &String) -> Transceiver {
        let socket_path = Path::new(path);
        let stream: UnixStream = UnixStream::connect(socket_path).unwrap();
        Transceiver {stream: stream, msg_map: HashMap::new()}
    }

    pub fn retrieve(&mut self, channel: &str) -> Vec<String> {
        let mut res: Vec<String> = vec![];
        if let Some(msgs) = self.msg_map.get_mut(channel) {
            std::mem::swap(msgs, &mut res);
        }
        res
    }
}