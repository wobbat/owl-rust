//! Error handling utilities

use std::process;

/// Print an error message and exit with code 1
pub fn exit_with_error(error: anyhow::Error) -> ! {
    eprintln!("{}", crate::internal::color::red(&error.to_string()));
    process::exit(1);
}
