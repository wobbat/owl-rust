mod cli;
mod commands;
mod core;
mod error;
mod internal;

fn main() {
    cli::handler::parse_and_execute();
}
