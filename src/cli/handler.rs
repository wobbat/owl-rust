use crate::commands::{add, adopt, apply, dots, edit, find};
use crate::internal::color;
use crate::internal::constants;
use clap::{Parser, Subcommand};

/// Global options for the CLI
#[derive(Debug, Clone, Parser)]
#[command(name = "owl", about = "Dotfile and package manager")]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Perform a dry run without making changes
    #[arg(long)]
    pub dry_run: bool,

    /// Run in non-interactive mode
    #[arg(short = 'y', long)]
    pub non_interactive: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Edit target types for better type safety
#[derive(Debug, Clone, PartialEq, clap::ValueEnum)]
pub enum EditTarget {
    Dots,
    Config,
}

/// Available commands for the CLI with better type safety
#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    /// Apply configuration (default command)
    Apply,
    /// Edit dotfiles or config
    Edit {
        /// Type to edit (dots or config)
        target: EditTarget,
        /// Argument for edit
        argument: String,
    },
    /// List dotfiles
    Dots,
    /// Add packages
    Add {
        /// Packages to add
        items: Vec<String>,
        /// Search mode
        #[arg(long)]
        search: bool,
    },
    /// Adopt existing packages
    Adopt {
        /// Packages to adopt
        items: Vec<String>,
        /// Adopt all packages
        #[arg(long)]
        all: bool,
    },
    /// Find packages or files
    Find {
        /// Query terms
        query: Vec<String>,
    },
    /// Check configuration
    ConfigCheck {
        /// Specific config file to check
        file: Option<String>,
    },
    /// Show host configuration
    ConfigHost,
    /// Clean up files
    Clean {
        /// Specific filename to clean
        filename: Option<String>,
    },
    /// Alias for edit dots
    #[command(alias = "de")]
    EditDots {
        /// Argument for edit
        argument: String,
    },
    /// Alias for edit config
    #[command(alias = "ce")]
    EditConfig {
        /// Argument for edit
        argument: String,
    },
}

/// Parsed command line options
#[derive(Debug, Clone)]
pub struct CliOptions {
    pub global: GlobalFlags,
    pub cmd: Command,
}

/// Global options for the CLI
#[derive(Debug, Clone)]
pub struct GlobalFlags {
    pub verbose: bool,
    pub dry_run: bool,
    pub non_interactive: bool,
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

/// Convert clap Commands to our Command enum
fn convert_command(cmd: Option<Commands>) -> Command {
    match cmd {
        Some(Commands::Apply) | None => Command::Apply,
        Some(Commands::Edit { target, argument }) => Command::Edit { target, argument },
        Some(Commands::Dots) => Command::Dots,
        Some(Commands::Add { items, search }) => Command::Add { items, search },
        Some(Commands::Adopt { items, all }) => Command::Adopt { items, all },
        Some(Commands::Find { query }) => Command::Find { query },
        Some(Commands::ConfigCheck { file }) => Command::ConfigCheck { file },
        Some(Commands::ConfigHost) => Command::ConfigHost,
        Some(Commands::Clean { filename }) => Command::Clean { filename },
        Some(Commands::EditDots { argument }) => Command::Edit { target: EditTarget::Dots, argument },
        Some(Commands::EditConfig { argument }) => Command::Edit { target: EditTarget::Config, argument },
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
pub fn parse_and_execute(_args: Vec<String>) {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            eprintln!("{}", color::red(&err.to_string()));
            std::process::exit(1);
        }
    };
    let opts = CliOptions {
        global: GlobalFlags {
            verbose: cli.verbose,
            dry_run: cli.dry_run,
            non_interactive: cli.non_interactive,
        },
        cmd: convert_command(cli.command),
    };
    execute_command(&opts);
}
