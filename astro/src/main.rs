use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from("kinetics.sock"))]
    kinetics_sock: String,
    #[arg(short, long, default_value_t = String::from("gps.sock"))]
    gps_sock: String,
    #[arg(short, long, default_value_t = String::from("messages.sock"))]
    messages_sock: String,
}

fn main() {
    let args = Args::parse();
    dbg!(args);
}
