use crate::commands::{add, adopt, apply, dots, edit, find};
use crate::internal::color;
use crate::internal::constants;
use anyhow::{anyhow, Result};
use phf::phf_map;
use std::sync::LazyLock;

/// Global options for the CLI
#[derive(Debug, Clone)]
pub struct GlobalFlags {
    pub verbose: bool,
    pub dry_run: bool,
    pub non_interactive: bool,
}

/// Edit target types for better type safety
#[derive(Debug, Clone, PartialEq)]
pub enum EditTarget {
    Dots,
    Config,
}

impl EditTarget {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            constants::EDIT_TYPE_DOTS => Some(Self::Dots),
            constants::EDIT_TYPE_CONFIG => Some(Self::Config),
            _ => None,
        }
    }
}

/// Available commands for the CLI with better type safety
#[derive(Debug, Clone)]
pub enum Command {
    Apply,
    Edit { target: EditTarget, argument: String },
    Dots,
    Add { items: Vec<String>, search: bool },
    Adopt { items: Vec<String>, all: bool },
    Find { query: Vec<String> },
    ConfigCheck { file: Option<String> },
    ConfigHost,
    Clean { filename: Option<String> },
}

// Command aliases mapping for cleaner alias resolution
static COMMAND_ALIASES: LazyLock<phf::Map<&'static str, (&'static str, &'static str)>> = LazyLock::new(|| {
    phf_map! {
        "de" => (constants::CMD_EDIT, constants::EDIT_TYPE_DOTS),
        "ce" => (constants::CMD_EDIT, constants::EDIT_TYPE_CONFIG),
    }
});

/// Parsed command line options
#[derive(Debug, Clone)]
pub struct CliOptions {
    pub global: GlobalFlags,
    pub cmd: Command,
}

/// Parse global flags (-v/--verbose, --dr, -y/--non-interactive) and return (verbose, dry_run, non_interactive, remaining_args)
pub fn parse_global_flags(args: &[String]) -> (bool, bool, bool, Vec<String>) {
    let mut verbose = false;
    let mut dry_run = false;
    let mut non_interactive = false;
    let mut filtered_args = Vec::new();
    for arg in args {
        if arg == "-v" || arg == "--verbose" {
            verbose = true;
        } else if arg == "--dr" {
            dry_run = true;
        } else if arg == "-y" || arg == "--non-interactive" {
            non_interactive = true;
        } else {
            filtered_args.push(arg.to_string());
        }
    }
    (verbose, dry_run, non_interactive, filtered_args)
}

/// Parse command from filtered arguments
pub fn parse_command(filtered_args: &[String]) -> Result<Command> {
    if filtered_args.is_empty() {
        return Ok(Command::Apply);
    }

    let command_name = &filtered_args[0];
    let command_arguments = &filtered_args[1..];

    // Handle aliases by mapping to their canonical commands
    let (canonical_cmd, mapped_args) = resolve_command_alias(command_name, command_arguments);

    parse_canonical_command(canonical_cmd, &mapped_args, command_name)
}

/// Resolve command aliases to their canonical form using static map
fn resolve_command_alias<'a>(command_name: &'a str, command_arguments: &[String]) -> (&'a str, Vec<String>) {
    if let Some((canonical_cmd, edit_type)) = COMMAND_ALIASES.get(command_name) {
        let mut alias_args = vec![edit_type.to_string()];
        alias_args.extend(command_arguments.iter().cloned());
        (canonical_cmd, alias_args)
    } else {
        (command_name, command_arguments.to_vec())
    }
}

/// Parse the canonical command and its arguments
fn parse_canonical_command(
    canonical_cmd: &str,
    mapped_args: &[String],
    original_cmd: &str,
) -> Result<Command> {
    match canonical_cmd {
        constants::CMD_APPLY => parse_no_args_command(mapped_args, || Command::Apply, "apply command takes no arguments"),
        constants::CMD_EDIT => parse_edit_command(mapped_args),
        constants::CMD_DOTS => parse_no_args_command(mapped_args, || Command::Dots, "dots command takes no arguments"),
        constants::CMD_ADD => parse_add_command(mapped_args),
        constants::CMD_ADOPT => parse_adopt_command(mapped_args),
        constants::CMD_FIND => parse_find_command(mapped_args),
        constants::CMD_CONFIGCHECK => parse_configcheck_command(mapped_args),
        constants::CMD_CONFIGHOST => parse_no_args_command(mapped_args, || Command::ConfigHost, "confighost command takes no arguments"),
        constants::CMD_CLEAN => parse_clean_command(mapped_args),
        _ => Err(anyhow!(
            "Unknown command: {}. Available commands: apply, edit, de, ce, dots, add, adopt, find, configcheck, confighost, clean",
            original_cmd
        )),
    }
}



/// Parse edit command with better type safety
fn parse_edit_command(args: &[String]) -> Result<Command> {
    if args.len() < 2 {
        return Err(anyhow!("edit command requires type and argument"));
    }

    let target = EditTarget::from_str(&args[0])
        .ok_or_else(|| anyhow!(
            "Invalid edit type: {}. Must be one of: dots, config",
            args[0]
        ))?;

    let argument = args[1..].join(" ");
    Ok(Command::Edit { target, argument })
}

/// Generic function to parse command arguments with optional flags
fn parse_args_with_optional_flag<T, F>(
    args: &[String],
    flag_parser: F,
    error_msg: String,
) -> Result<(Vec<String>, T)>
where
    F: Fn(&str) -> Option<T>,
    T: Default,
{
    if args.is_empty() {
        return Err(anyhow::anyhow!(error_msg));
    }

    let mut flags = T::default();
    let mut items = Vec::new();

    for arg in args {
        if let Some(flag_value) = flag_parser(arg) {
            flags = flag_value;
        } else {
            items.push(arg.to_string());
        }
    }

    if items.is_empty() {
        return Err(anyhow::anyhow!(error_msg));
    }

    Ok((items, flags))
}

/// Parse add command
fn parse_add_command(args: &[String]) -> Result<Command> {
    let (items, search_mode) = parse_args_with_optional_flag(
        args,
        |arg| if arg == "--search" { Some(true) } else { None },
        "add command requires at least one item".to_string(),
    )?;

    Ok(Command::Add {
        items,
        search: search_mode,
    })
}

/// Parse adopt command
fn parse_adopt_command(args: &[String]) -> Result<Command> {
    let (items, all) = parse_args_with_optional_flag(
        args,
        |arg| if arg == "--all" { Some(true) } else { None },
        "adopt requires at least one package or --all".to_string(),
    )?;

    // For adopt command, --all flag means no items are required
    if !all && items.is_empty() {
        return Err(anyhow!("adopt requires at least one package or --all"));
    }

    Ok(Command::Adopt { items, all })
}

/// Parse configcheck command
fn parse_configcheck_command(args: &[String]) -> Result<Command> {
    parse_optional_single_arg_command(
        args,
        || Command::ConfigCheck { file: None },
        |file| Command::ConfigCheck { file: Some(file) },
        "configcheck command takes at most one .owl file argument",
    )
}



/// Parse clean command
fn parse_clean_command(args: &[String]) -> Result<Command> {
    parse_optional_single_arg_command(
        args,
        || Command::Clean { filename: None },
        |filename| Command::Clean { filename: Some(filename) },
        "clean command takes at most one filename",
    )
}

/// Parse find command
fn parse_find_command(args: &[String]) -> Result<Command> {
    if args.is_empty() {
        return Err(anyhow!("find command requires at least one argument"));
    }

    Ok(Command::Find {
        query: args.to_vec(),
    })
}

/// Generic helper to parse commands that take no arguments
fn parse_no_args_command<F>(args: &[String], command_constructor: F, error_msg: &str) -> Result<Command>
where
    F: FnOnce() -> Command,
{
    if args.is_empty() {
        Ok(command_constructor())
    } else {
        Err(anyhow!("{}", error_msg))
    }
}

/// Generic helper to parse commands that take at most one argument
fn parse_optional_single_arg_command<F, T>(
    args: &[String],
    none_constructor: F,
    some_constructor: fn(String) -> T,
    error_msg: &str,
) -> Result<T>
where
    F: FnOnce() -> T,
{
    match args.len() {
        0 => Ok(none_constructor()),
        1 => Ok(some_constructor(args[0].to_string())),
        _ => Err(anyhow!("{}", error_msg)),
    }
}



/// Execute the parsed command
pub fn execute_command(opts: &CliOptions) {
    if opts.global.verbose {
        println!("{}", color::dim("[verbose] args parsed"));
    }
    match &opts.cmd {
        Command::Apply => apply::run(opts),
        Command::Edit { target, argument } => {
            let typ = match target {
                EditTarget::Dots => constants::EDIT_TYPE_DOTS,
                EditTarget::Config => constants::EDIT_TYPE_CONFIG,
            };
            if let Err(err) = edit::run(typ, argument) {
                eprintln!("{}", color::red(&err.to_string()));
                std::process::exit(1);
            }
        }
        Command::Dots => dots::run(opts),
        Command::Add { items, search } => add::run(items, *search),
        Command::Adopt { items, all } => adopt::run(items, *all),
        Command::Find { query } => find::run(query),
        Command::ConfigCheck { file } => {
            if let Some(f) = file {
                if let Err(err) = crate::core::config::validator::run_configcheck(f) {
                    eprintln!("{}", color::red(&err.to_string()));
                    std::process::exit(1);
                }
            } else {
                if let Err(err) = crate::core::config::validator::run_full_configcheck() {
                    eprintln!("{}", color::red(&err.to_string()));
                    std::process::exit(1);
                }
            }
        }
        Command::ConfigHost => {
            if let Err(err) = crate::core::config::validator::run_confighost() {
                eprintln!("{}", color::red(&err.to_string()));
                std::process::exit(1);
            }
        }
        Command::Clean { filename } => {
            let result = match filename {
                Some(fname) => {
                    let result = crate::commands::clean::handle_clean(fname);
                    if result.is_ok() {
                        println!("[{}]", color::blue("clean"));
                        println!("  {} {}", color::green("✓"), color::dim(fname));
                    }
                    result
                },
                None => crate::commands::clean::handle_clean_all(),
            };
            if let Err(err) = result {
                eprintln!("{}", color::red(&err.to_string()));
                std::process::exit(1);
            }
        }
    }
}

/// Parse command line arguments and execute the corresponding command
pub fn parse_and_execute(args: Vec<String>) {
    let (verbose, dry_run, non_interactive, filtered_args) = parse_global_flags(&args);
    let cmd = match parse_command(&filtered_args) {
        Ok(cmd) => cmd,
        Err(err) => {
            eprintln!("{}", color::red(&err.to_string()));
            std::process::exit(1);
        }
    };
    let opts = CliOptions {
        global: GlobalFlags {
            verbose,
            dry_run,
            non_interactive,
        },
        cmd,
    };
    execute_command(&opts);
}
