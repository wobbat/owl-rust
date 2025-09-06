//! Error handling utilities

use std::process;

#[derive(Debug)]
pub enum OwlError {
    InvalidArguments(String),
    IoError(std::io::Error),
}

impl std::fmt::Display for OwlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OwlError::InvalidArguments(msg) => write!(f, "Invalid arguments: {}", msg),
            OwlError::IoError(err) => write!(f, "IO error: {}", err),
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
    eprintln!("{}", crate::internal::color::red(message));
    process::exit(1);
}

// removed unused handle_result
