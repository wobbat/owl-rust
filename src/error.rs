//! Error handling utilities

use std::process;

#[derive(Debug)]
pub enum OwlError {
    NoCommandProvided,
    InvalidArguments(String),
    ConfigFileNotFound,
    EditTypeInvalid,
    PackageManagerError(String),
    IoError(std::io::Error),
    ParseError(String),
}

impl std::fmt::Display for OwlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OwlError::NoCommandProvided => write!(f, "No command provided"),
            OwlError::InvalidArguments(msg) => write!(f, "Invalid arguments: {}", msg),
            OwlError::ConfigFileNotFound => write!(f, "Config file not found"),
            OwlError::EditTypeInvalid => write!(f, "Edit type must be dots or config"),
            OwlError::PackageManagerError(msg) => write!(f, "Package manager error: {}", msg),
            OwlError::IoError(err) => write!(f, "IO error: {}", err),
            OwlError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for OwlError {}

impl From<std::io::Error> for OwlError {
    fn from(err: std::io::Error) -> Self {
        OwlError::IoError(err)
    }
}

/// Print an error message and exit with code 1
pub fn exit_with_error(message: &str) -> ! {
    eprintln!("{}", crate::colo::red(message));
    process::exit(1);
}

/// Handle a result, printing error and exiting on failure
pub fn handle_result<T>(result: Result<T, String>, context: &str) -> T {
    match result {
        Ok(value) => value,
        Err(err) => {
            eprintln!("{}", crate::colo::red(&format!("{}: {}", context, err)));
            process::exit(1);
        }
    }
}

/// Print a success message
#[allow(dead_code)]
pub fn print_success(message: &str) {
    println!("{}", crate::colo::green(message));
}

/// Print an info message
#[allow(dead_code)]
pub fn print_info(message: &str) {
    println!("{}", crate::colo::blue(message));
}

/// Print a warning message
#[allow(dead_code)]
pub fn print_warning(message: &str) {
    println!("{}", crate::colo::yellow(message));
}