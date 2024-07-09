use std::collections::HashMap;
use std::io::{BufReader, BufWriter};
use std::os::unix::net::UnixStream;
use std::path::Path;

pub struct Transceiver {
    stream: UnixStream,
    msg_map: HashMap<String, Vec<String>>,
}

impl Transceiver {
    pub fn new(path: &String) -> Transceiver {
        let socket_path = Path::new(path);
        let stream: UnixStream = UnixStream::connect(socket_path).unwrap();
        Transceiver {stream: stream, msg_map: HashMap::new()}
    }

    pub fn retrieve<T>(&mut self, channel: &str) -> Vec<T> {
        let mut msg_vec: Vec<String> = vec![];
        if let Some(old_msg_vec) = self.msg_map.get_mut(channel) {
            std::mem::swap(old_msg_vec, &mut msg_vec);
        }
        let mut res: Vec<T> = vec![];
        for msg in msg_vec {}
        res
    }
}