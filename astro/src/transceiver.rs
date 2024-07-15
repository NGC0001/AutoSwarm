mod io_rc {

use std::io::{self, prelude::*};
use std::rc::Rc;

#[derive(Debug)]
pub struct IoRc<T>(Rc<T>);

impl<T> IoRc<T> {
    pub fn from(rc: &Rc<T>) -> Self {
        Self(rc.clone())
    }
}

impl<T> Read for IoRc<T>
where for<'a> &'a T: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (&mut &*self.0).read(buf)
    }
}

impl<T> Write for IoRc<T>
where for<'a> &'a T: Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (&mut &*self.0).write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        (&mut &*self.0).flush()
    }
}

}

use std::collections::HashMap;
use std::io::{BufReader, BufWriter};
use std::io::prelude::*;
use std::net::Shutdown;
use std::ops::Drop;
use std::os::unix::net::UnixStream;
use std::rc::Rc;

use serde::de::DeserializeOwned;
use serde::Serialize;

use io_rc::IoRc;

pub struct Transceiver {
    stream: Rc<UnixStream>,
    writer: BufWriter<IoRc<UnixStream>>,
    reader: BufReader<IoRc<UnixStream>>,
    readbuf: Vec<u8>,
    msg_map: HashMap<String, Vec<String>>,
}

impl Drop for Transceiver {
    fn drop(&mut self) {
        (*self.stream).shutdown(Shutdown::Both).unwrap();
    }
}

impl Transceiver {
    pub fn new(stream: UnixStream) -> Transceiver {
        let s = Rc::new(stream);
        Transceiver {
            stream: s.clone(),
            writer: BufWriter::new(IoRc::from(&s)),
            reader: BufReader::new(IoRc::from(&s)),
            readbuf: vec![],
            msg_map: HashMap::new(),
        }
    }

    pub fn retrieve_raw(&mut self, channel: &str) -> Vec<String> {
        self.do_receive();
        let mut msg_vec: Vec<String> = vec![];
        if let Some(old_msg_vec) = self.msg_map.get_mut(channel) {
            std::mem::swap(old_msg_vec, &mut msg_vec);
        }
        msg_vec
    }

    pub fn retrieve<T>(&mut self, channel: &str) -> Vec<T>
    where T: DeserializeOwned,
    {
        let mut res: Vec<T> = vec![];
        for msg in self.retrieve_raw(channel) {
            res.push(serde_json::from_str(&msg).unwrap());
        }
        res
    }

    pub fn send_raw(&mut self, channel: &str, data: &String) {
        let len_bytes: [u8; 4] = (data.len() as u32).to_le_bytes();
        self.writer.write_all(channel.as_bytes()).unwrap();
        self.writer.write_all(&len_bytes).unwrap();
        self.writer.write_all(data.as_bytes()).unwrap();
        self.writer.flush().unwrap();
    }

    pub fn send<T>(&mut self, channel: &str, v: &T)
    where T: Serialize {
        let data = serde_json::to_string(v).unwrap();
        dbg!(&data);
        self.send_raw(channel, &data);
    }

    fn do_receive(&mut self) {
        let mut buf: Vec<u8> = vec![];
        self.reader.read(&mut buf).unwrap();
        self.readbuf.append(&mut buf);
        loop {
            if self.readbuf.len() < 8 {
                break;
            }
            let len_bytes: [u8; 4] = <[u8; 4]>::try_from(&self.readbuf[4..8]).unwrap();
            let len: usize = u32::from_le_bytes(len_bytes) as usize;
            if self.readbuf.len() - 8 < len {
                break;
            }
            let channel: &str = std::str::from_utf8(&self.readbuf[..4]).unwrap();
            let data: &str = std::str::from_utf8(&self.readbuf[8..][..len]).unwrap();
            dbg!(&data);
            self.msg_map.entry(String::from(channel)).or_insert(vec![]).push(String::from(data));
            self.readbuf.drain(..(len + 8));
        }
    }
}