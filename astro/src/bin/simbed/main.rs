use clap::Parser;

mod uavsim;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    astro_bin: String,
    #[arg(long, default_value_t = 1)]
    num_uav: u32,
}

fn main() {
    println!("Hello, world!");
}