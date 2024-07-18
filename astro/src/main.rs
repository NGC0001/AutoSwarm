use clap::Parser;

use astro::{Astro, astroconf::AstroConf};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    id: u32,
    #[arg(long)]
    uav_radius: f32,
}

fn main() {
    let args = Args::parse();
    let conf = AstroConf {
        id: args.id,
        uav_radius: args.uav_radius,
    };
    conf.validate().unwrap();
    let mut astro = Astro::new(conf);
    astro.init();
    astro.run_event_loop();
}