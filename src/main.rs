use std::env as std_env;

mod add;
mod apply;
mod cmd_handler;
mod colo;
mod config;
mod constants;
mod dotfiles;
mod dots;
mod edit;
mod env;
mod error;
mod files;
mod package;
mod services;
mod state;
mod ui;
mod util;

fn main() {
    let args: Vec<String> = std_env::args().skip(1).collect();

    // Special case for testing uninstalled packages
    if args.len() == 1 && args[0] == "uninstalled" {
        handle_uninstalled_command();
        return;
    }

    cmd_handler::parse_and_execute(args);
}

/// Handle the 'uninstalled' command to show packages that are not installed
fn handle_uninstalled_command() {
    match config::Config::load_all_relevant_config_files() {
        Ok(config) => {
            match config::get_uninstalled_packages(&config) {
                Ok(uninstalled) => {
                    if uninstalled.is_empty() {
                        println!("All packages are installed!");
                    } else {
                        println!("Uninstalled packages:");
                        for package in uninstalled {
                            println!("  {}", package);
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Error checking package status: {}", err);
                    std::process::exit(1);
                }
            }
        }
        Err(err) => {
            eprintln!("Error loading config: {}", err);
            std::process::exit(1);
        }
    }
}
