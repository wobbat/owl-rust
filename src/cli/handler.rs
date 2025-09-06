use crate::commands::{add, adopt, apply, dots, edit};
use crate::infrastructure::color as colo;
use crate::infrastructure::constants;

/// Global options for the CLI
#[derive(Debug, Clone)]
pub struct GlobalFlags {
    pub verbose: bool,
    pub dry_run: bool,
}

/// Available commands for the CLI
#[derive(Debug, Clone)]
pub enum Command {
    Apply { dry_run: bool },
    Edit { typ: String, arg: String },
    Dots { dry_run: bool },
    Add { items: Vec<String>, search: bool },
    Adopt { items: Vec<String>, all: bool },
    ConfigCheck { file: String },
    ConfigHost,
}

/// Parsed command line options
#[derive(Debug, Clone)]
pub struct CliOptions {
    pub global: GlobalFlags,
    pub cmd: Command,
}

/// Parse global flags (-v/--verbose, --dr) and return (verbose, dry_run, remaining_args)
pub fn parse_global_flags(args: &[String]) -> (bool, bool, Vec<String>) {
    let mut verbose = false;
    let mut dry_run = false;
    let mut filtered_args = Vec::new();
    for arg in args {
        if arg == "-v" || arg == "--verbose" {
            verbose = true;
        } else if arg == "--dr" {
            dry_run = true;
        } else {
            filtered_args.push(arg.clone());
        }
    }
    (verbose, dry_run, filtered_args)
}

/// Parse command from filtered arguments
pub fn parse_command(filtered_args: &[String]) -> Result<Command, crate::error::OwlError> {
    if filtered_args.is_empty() {
        return Ok(Command::Apply { dry_run: false });
    }

    let cmd_str = &filtered_args[0];
    let cmd_args = &filtered_args[1..];

    // Handle aliases by mapping to their canonical commands
    let (canonical_cmd, mapped_args) = resolve_command_alias(cmd_str, cmd_args);

    parse_canonical_command(canonical_cmd, &mapped_args, cmd_str)
}

/// Resolve command aliases to their canonical form
fn resolve_command_alias<'a>(cmd_str: &'a str, cmd_args: &[String]) -> (&'a str, Vec<String>) {
    match cmd_str {
        constants::CMD_DE => (constants::CMD_EDIT, vec![
            constants::EDIT_TYPE_DOTS.to_string(),
            cmd_args.join(" ")
        ]),
        constants::CMD_CE => (constants::CMD_EDIT, vec![
            constants::EDIT_TYPE_CONFIG.to_string(),
            cmd_args.join(" ")
        ]),
        _ => (cmd_str, cmd_args.to_vec())
    }
}

/// Parse the canonical command and its arguments
fn parse_canonical_command(canonical_cmd: &str, mapped_args: &[String], original_cmd: &str) -> Result<Command, crate::error::OwlError> {
    match canonical_cmd {
        constants::CMD_APPLY => parse_apply_command(mapped_args),
        constants::CMD_EDIT => parse_edit_command(mapped_args),
        constants::CMD_DOTS => parse_dots_command(mapped_args),
        constants::CMD_ADD => parse_add_command(mapped_args),
        constants::CMD_ADOPT => parse_adopt_command(mapped_args),
        constants::CMD_CONFIGCHECK => parse_configcheck_command(mapped_args),
        constants::CMD_CONFIGHOST => parse_confighost_command(mapped_args),
        _ => Err(crate::error::OwlError::InvalidArguments(format!(
            "Unknown command: {}. Available commands: apply, edit, de, ce, dots, add, adopt, configcheck, confighost",
            original_cmd
        ))),
    }
}

/// Parse apply command
fn parse_apply_command(args: &[String]) -> Result<Command, crate::error::OwlError> {
    ensure_no_args(args, "apply command takes no arguments")?;
    Ok(Command::Apply { dry_run: false })
}

/// Parse dots command
fn parse_dots_command(args: &[String]) -> Result<Command, crate::error::OwlError> {
    ensure_no_args(args, "dots command takes no arguments")?;
    Ok(Command::Dots { dry_run: false })
}

/// Parse edit command
fn parse_edit_command(args: &[String]) -> Result<Command, crate::error::OwlError> {
    if args.len() >= 2 {
        let typ = args[0].clone();
        let arg = args[1..].join(" ");
        Ok(Command::Edit { typ, arg })
    } else {
        Err(crate::error::OwlError::InvalidArguments("edit command requires type and argument".to_string()))
    }
}

/// Parse add command
fn parse_add_command(args: &[String]) -> Result<Command, crate::error::OwlError> {
    if args.is_empty() {
        return Err(crate::error::OwlError::InvalidArguments("add command requires at least one item".to_string()));
    }

    // Check for --search flag
    let mut search_mode = false;
    let mut items = Vec::new();

    for arg in args {
        if arg == "--search" {
            search_mode = true;
        } else {
            items.push(arg.clone());
        }
    }

    if items.is_empty() {
        return Err(crate::error::OwlError::InvalidArguments("add command requires at least one item".to_string()));
    }

    Ok(Command::Add { items, search: search_mode })
}

/// Parse adopt command
fn parse_adopt_command(args: &[String]) -> Result<Command, crate::error::OwlError> {
    if args.is_empty() {
        return Err(crate::error::OwlError::InvalidArguments(
            "adopt requires package names or --all".to_string(),
        ));
    }

    let mut all = false;
    let mut items = Vec::new();
    for arg in args {
        if arg == "--all" { all = true; } else { items.push(arg.clone()); }
    }

    if !all && items.is_empty() {
        return Err(crate::error::OwlError::InvalidArguments(
            "adopt requires at least one package or --all".to_string(),
        ));
    }

    Ok(Command::Adopt { items, all })
}

/// Parse configcheck command
fn parse_configcheck_command(args: &[String]) -> Result<Command, crate::error::OwlError> {
    if args.len() == 1 {
        Ok(Command::ConfigCheck { file: args[0].clone() })
    } else {
        Err(crate::error::OwlError::InvalidArguments("configcheck command requires exactly one .owl file argument".to_string()))
    }
}

/// Parse confighost command
fn parse_confighost_command(args: &[String]) -> Result<Command, crate::error::OwlError> {
    if args.is_empty() {
        Ok(Command::ConfigHost)
    } else {
        Err(crate::error::OwlError::InvalidArguments("confighost command takes no arguments".to_string()))
    }
}

fn ensure_no_args(args: &[String], message: &str) -> Result<(), crate::error::OwlError> {
    if args.is_empty() {
        Ok(())
    } else {
        Err(crate::error::OwlError::InvalidArguments(message.to_string()))
    }
}

/// Execute the parsed command
pub fn execute_command(opts: &CliOptions) {
    if opts.global.verbose {
        println!("{}", colo::dim("[verbose] args parsed"));
    }
    match &opts.cmd {
        Command::Apply { dry_run } => apply::run(*dry_run || opts.global.dry_run),
        Command::Edit { typ, arg } => {
            if let Err(err) = edit::run(typ, arg) {
                eprintln!("{}", colo::red(&err));
                std::process::exit(1);
            }
        }
        Command::Dots { dry_run } => dots::run(*dry_run || opts.global.dry_run),
        Command::Add { items, search } => add::run(items, *search),
        Command::Adopt { items, all } => adopt::run(items, *all),
        Command::ConfigCheck { file } => {
            if let Err(err) = crate::domain::config::run_configcheck(file) {
                eprintln!("{}", colo::red(&err.to_string()));
                std::process::exit(1);
            }
        }
        Command::ConfigHost => {
            if let Err(err) = crate::domain::config::run_confighost() {
                eprintln!("{}", colo::red(&err.to_string()));
                std::process::exit(1);
            }
        }
    }
}

/// Parse command line arguments and execute the corresponding command
pub fn parse_and_execute(args: Vec<String>) {
    let (verbose, dry_run, filtered_args) = parse_global_flags(&args);
    let cmd = match parse_command(&filtered_args) {
        Ok(cmd) => cmd,
        Err(err) => {
            eprintln!("{}", colo::red(&err.to_string()));
            std::process::exit(1);
        }
    };
    let opts = CliOptions {
        global: GlobalFlags { verbose, dry_run },
        cmd,
    };
    execute_command(&opts);
}
