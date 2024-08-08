use clap::Parser;

use astro::{Astro, AstroConf};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    id: u32,
    #[arg(long)]
    uav_radius: f32,
    #[arg(long)]
    msg_range: f32,
    #[arg(long, default_value_t = 0.95)]
    contact_range_ratio: f32,  // [0.0, 1.0]
    #[arg(long)]
    max_v: f32,
}

fn main() {
    let args = Args::parse();
    let conf = AstroConf {
        id: args.id,
        uav_radius: args.uav_radius,
        msg_range: args.msg_range,
        contact_range: args.msg_range * args.contact_range_ratio,
        max_v: args.max_v,
    };
    conf.validate().unwrap();
    let mut astro = Astro::new(conf);
    astro.init();
    astro.run_event_loop();
}