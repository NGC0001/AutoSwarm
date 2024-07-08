use clap::Parser;

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
