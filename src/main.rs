use std::env;

mod colo;
mod apply;
mod edit;
mod add;
mod ui;
mod cmd_handler;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    cmd_handler::parse_and_execute(args);
}
