use clap::Parser;
use std::{cell::RefCell, rc::Rc};

mod control;
mod gps;
mod transceiver;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from("socket"))]
    socket_file: String,
}

fn main() {
    let args = Args::parse();
    dbg!(args);
}
