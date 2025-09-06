use std::env as std_env;

mod cli;
mod commands;
mod core;
mod error;
mod internal;

fn main() {
    let args: Vec<String> = std_env::args().skip(1).collect();
    cli::handler::parse_and_execute(args);
}
