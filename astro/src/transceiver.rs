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
use std::io::{self, BufReader, BufWriter};
use std::io::prelude::*;
use std::net::Shutdown;
use std::ops::Drop;
use std::os::unix::net::UnixStream;
use std::rc::Rc;
use std::result::Result;
use std::thread;
use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::Serialize;

use io_rc::IoRc;

pub const SEND_RETRY_INTERVAL: Duration = Duration::from_millis(20);

// channel meta field
pub const META_CHANNEL_B: usize = 0;
pub const META_CHANNEL_SZ: usize = 4;
pub const META_CHANNEL_E: usize = META_CHANNEL_B + META_CHANNEL_SZ;
// length meta field
pub const META_LENGTH_B: usize = META_CHANNEL_E;
pub const META_LENGTH_SZ: usize = 4;
pub const META_LENGTH_E: usize = META_LENGTH_B + META_LENGTH_SZ;
// all meta fields
pub const META_SZ: usize = META_LENGTH_E;

pub struct Transceiver {
    stream: Rc<UnixStream>,
    writer: BufWriter<IoRc<UnixStream>>,
    reader: BufReader<IoRc<UnixStream>>,
    cache: Vec<u8>,
    msg_map: HashMap<String, Vec<String>>,
}

impl Drop for Transceiver {
    fn drop(&mut self) {
        (*self.stream).shutdown(Shutdown::Both).unwrap();
    }
}

impl Transceiver {
    pub fn new(stream: UnixStream) -> Transceiver {
        stream.set_nonblocking(true).unwrap();
        let s = Rc::new(stream);
        Transceiver {
            stream: s.clone(),
            writer: BufWriter::new(IoRc::from(&s)),
            reader: BufReader::new(IoRc::from(&s)),
            cache: vec![],
            msg_map: HashMap::new(),
        }
    }

    pub fn retrieve_raw(&mut self, channel: &str) -> Vec<String> {
        self.do_receive().unwrap();
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

    // this function may block
    pub fn send_raw(&mut self, channel: &str, data: &String) {
        assert!(channel.len() == META_CHANNEL_SZ,
            "channel string must be {} bytes long", META_CHANNEL_SZ);
        let len_bytes: [u8; META_LENGTH_SZ] = (data.len() as u32).to_le_bytes();
        self.writer.write_all(channel.as_bytes()).unwrap();
        self.writer.write_all(&len_bytes).unwrap();
        self.writer.write_all(data.as_bytes()).unwrap();
        let mut num_flush_try = 0;
        while match self.writer.flush() {
            Ok(..) => Ok(false),  // successfully flushed, no more loops
            Err(e) => match e.kind() {
                io::ErrorKind::WouldBlock => Ok(true),  // busy resource, try again, may block
                _ => Err(e),  // true error
            }
        }.unwrap() {
            num_flush_try += 1;
            println!("redo send {}", num_flush_try);
            thread::sleep(SEND_RETRY_INTERVAL);
        }
    }

    pub fn send<T>(&mut self, channel: &str, v: &T)
    where T: Serialize {
        let data = serde_json::to_string(v).unwrap();
        self.send_raw(channel, &data);
    }

    // buf-reading should not block (should use nonblocking io)
    // note,
    // `BufReader::read` reads until EOF,
    // `BufReader::read_until` reads until specified delimiter.
    fn do_receive(&mut self) -> Result<(), io::Error> {
        match self.reader.fill_buf() {
            Ok(buf) => {
                let nbytes = buf.len();
                Self::pick_data(&mut self.msg_map, &mut self.cache, buf);
                self.reader.consume(nbytes);
                Ok(())
            },
            Err(e) => match e.kind() {
                io::ErrorKind::WouldBlock => Ok(()),
                _ => Err(e)
            },
        }
    }

    fn pick_data(msg_map: &mut HashMap<String, Vec<String>>, cache: &mut Vec<u8>, buf: &[u8]) {
        let mut bytes_read: usize = Self::pick_data_with_cache(msg_map, cache, buf);
        debug_assert!(cache.is_empty() || bytes_read == buf.len());
        loop {
            let bytes_left: usize = buf.len() - bytes_read;
            if bytes_left < META_SZ {
                break;
            }
            let meta_slice: &[u8] = &buf[bytes_read .. (bytes_read + META_SZ)];
            let (channel, len) = Self::get_channel_and_length_from_slice(meta_slice);
            if bytes_left < META_SZ + len {
                break;
            }
            let data_bytes_slice: &[u8] = &buf[(bytes_read + META_SZ) .. (bytes_read + META_SZ + len)];
            let data: &str = std::str::from_utf8(data_bytes_slice).unwrap();
            msg_map.entry(String::from(channel)).or_insert(vec![]).push(String::from(data));
            bytes_read += META_SZ + len;
        }
        if bytes_read < buf.len() {
            cache.extend_from_slice(&buf[bytes_read..]);
        }
    }

    // this function does one more copy than `pick_data`
    fn pick_data_with_cache(msg_map: &mut HashMap<String, Vec<String>>, cache: &mut Vec<u8>, buf: &[u8]) -> usize {
        if cache.is_empty() {
            return 0;
        }
        if cache.len() + buf.len() < META_SZ {
            cache.extend_from_slice(buf);
            return buf.len();
        }
        let mut bytes_read: usize = 0;
        if cache.len() < META_SZ {
            let meta_bytes_in_buf: usize = META_SZ - cache.len();
            cache.extend_from_slice(&buf[..meta_bytes_in_buf]);
            bytes_read += meta_bytes_in_buf;
        }
        let (channel, len) = Self::get_channel_and_length_from_slice(&cache[..META_SZ]);
        if cache.len() + buf.len() - bytes_read < META_SZ + len {
            cache.extend_from_slice(&buf[bytes_read..]);
            return buf.len();
        }
        let channel_str = String::from(channel);
        let data_bytes_in_buf: usize = META_SZ + len - cache.len();
        cache.extend_from_slice(&buf[bytes_read .. (bytes_read + data_bytes_in_buf)]);
        bytes_read += data_bytes_in_buf;
        let data: &str = std::str::from_utf8(&cache[META_SZ..]).unwrap();
        msg_map.entry(channel_str).or_insert(vec![]).push(String::from(data));
        cache.clear();
        bytes_read
    }

    fn get_channel_and_length_from_slice<'a>(buf: &'a[u8]) -> (&'a str, usize) {
        let channel_bytes_slice: &[u8] = &buf[META_CHANNEL_B..META_CHANNEL_E];
        let channel: &str = std::str::from_utf8(channel_bytes_slice).unwrap();
        let len_bytes_slice: &[u8] = &buf[META_LENGTH_B..META_LENGTH_E];
        let len_bytes: [u8; META_LENGTH_SZ] = <[u8; META_LENGTH_SZ]>::try_from(len_bytes_slice).unwrap();
        let len: usize = u32::from_le_bytes(len_bytes) as usize;
        (channel, len)
    }
}