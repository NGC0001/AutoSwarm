use clap::Parser;

mod uavsim;
mod uav;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    astro_bin: String,
    #[arg(long, default_value_t = 1)]
    num_uav: u32,
}

fn main() {
    let args = Args::parse();
    let mut uavs: Vec<uav::Uav> = vec![];
    for id in 0..args.num_uav {
        uavs.push(uav::Uav::new(id));
    }
}