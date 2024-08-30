# Autonomous Hierarchical Control and Coordination of Unmanned Aerial Vehicle Swarms

This is a Rust workspace with two packages `astro` and `quantity`.
"astro" stands for "Autonomous Swarm Tasking Routing and Organisation".

## Usage

1. [Install the Rust environment](https://www.rust-lang.org/tools/install).
2. Clone this repo: `git clone https://github.com/NGC0001/AutoSwarm.git`.
3. Change to the cloned project directory: `cd AutoSwarm`.
4. Build the release version of `astro`: `cargo build --release --bin astro`
5. Build simbed and run built-in demo: `cargo run --release --bin simbed -- --num-uav 10 --task-book demo_simple_line`
6. Press Ctrl-C to stop running.
7. The output file is `output/out-YYYYMMDD-HHMMSS`, with each line a json object.
8. Visualise the `(idx+1)`-th json: `python3 output/out-YYYYMMDD-HHMMSS idx`.
The visualisation needs python package `matplotlib` and `networkx`.

Note, if the number of UAVs specified by the `--num-uav` argument is too large,
the program may crash.
An empirical rule is less than two time the number of the cpu cores.
