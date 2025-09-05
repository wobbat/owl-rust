use crate::colo;
use crate::apply;
use crate::edit;
use crate::add;

#[derive(Debug, Clone)]
pub struct Global {
    pub verbose: bool,
}

#[derive(Debug, Clone)]
pub enum Command {
    Apply,
    Edit { typ: String, arg: String },
    Add { items: Vec<String> },
}

#[derive(Debug, Clone)]
pub struct Opts {
    pub global: Global,
    pub cmd: Command,
}

pub fn parse_verbose(args: &[String]) -> (bool, Vec<String>) {
    let mut verbose = false;
    let mut filtered_args = Vec::new();
    for arg in args {
        if arg == "-v" || arg == "--verbose" {
            verbose = true;
        } else {
            filtered_args.push(arg.clone());
        }
    }
    (verbose, filtered_args)
}

pub fn parse_command(filtered_args: &[String]) -> Command {
    if filtered_args.is_empty() {
        crate::ui::print_usage();
        std::process::exit(1);
    }

    let cmd_str = &filtered_args[0];
    let cmd_args = &filtered_args[1..];

    match cmd_str.as_str() {
        "apply" => {
            if !cmd_args.is_empty() {
                eprintln!("{}", colo::red("apply command takes no arguments"));
                std::process::exit(1);
            }
            Command::Apply
        }
        "edit" => {
            if cmd_args.len() < 2 {
                eprintln!(
                    "{}",
                    colo::red("edit command requires type and argument")
                );
                std::process::exit(1);
            }
            let typ = cmd_args[0].clone();
            let arg = cmd_args[1..].join(" ");
            Command::Edit { typ, arg }
        }
        "de" => {
            if cmd_args.is_empty() {
                eprintln!("{}", colo::red("de requires an argument"));
                std::process::exit(1);
            }
            let arg = cmd_args.join(" ");
            Command::Edit { typ: "dots".to_string(), arg }
        }
        "ce" => {
            if cmd_args.is_empty() {
                eprintln!("{}", colo::red("ce requires an argument"));
                std::process::exit(1);
            }
            let arg = cmd_args.join(" ");
            Command::Edit { typ: "config".to_string(), arg }
        }
        "add" => {
            if cmd_args.is_empty() {
                eprintln!("{}", colo::red("add command requires at least one item"));
                std::process::exit(1);
            }
            Command::Add {
                items: cmd_args.to_vec(),
            }
        }
        _ => {
            eprintln!("{}", colo::red(&format!("Unknown command: {}", cmd_str)));
            eprintln!("{}", colo::yellow("Available commands: apply, edit, de, ce, add"));
            std::process::exit(1);
        }
    }
}

pub fn execute_command(opts: &Opts) {
    if opts.global.verbose {
        println!("{}", colo::dim("[verbose] args parsed"));
    }
    match &opts.cmd {
        Command::Apply => apply::run(),
        Command::Edit { typ, arg } => edit::run(typ, arg),
        Command::Add { items } => add::run(items),
    }
}

pub fn parse_and_execute(args: Vec<String>) {
    let (verbose, filtered_args) = parse_verbose(&args);
    let cmd = parse_command(&filtered_args);
    let opts = Opts {
        global: Global { verbose },
        cmd,
    };
    execute_command(&opts);
}