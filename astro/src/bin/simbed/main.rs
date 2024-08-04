use clap::Parser;

mod gcs;
mod simbed;
mod uavconf;
mod uavsim;
mod uav;

use simbed::SimBed;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t = String::from("target/debug/astro"))]
    astro_bin: String,
    #[arg(long, default_value_t = 4)]
    num_uav: u32,
    #[arg(long, default_value_t = String::new())]
    task_book: String,
}

fn main() {
    let args = Args::parse();
    let mut simbed = SimBed::new(args.num_uav, &args.astro_bin, &args.task_book);
    simbed.run_sim_loop();
}