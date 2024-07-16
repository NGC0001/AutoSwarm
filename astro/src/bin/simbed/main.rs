use clap::Parser;

mod uav;
mod uavsim;
mod simbed;

use simbed::SimBed;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t = String::from("target/debug/astro"))]
    astro_bin: String,
    #[arg(long, default_value_t = 4)]
    num_uav: u32,
}

fn main() {
    let args = Args::parse();
    let mut simbed = SimBed::new(args.num_uav, &args.astro_bin);
    simbed.run_sim_loop();
}