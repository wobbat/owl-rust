use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// Shared spinner frames so all spinners look consistent
const SPINNER_FRAMES: &[&str] = &["⁚", "⁖", "⁘", "⁛", "⁙", "⁛", "⁘", "⁖"];

fn spinner_print_frame(message: &str, frame_index: usize) {
    print!(
        "\r\x1b[2K  {} {}...",
        crate::internal::color::blue(SPINNER_FRAMES[frame_index % SPINNER_FRAMES.len()]),
        message
    );
    io::stdout().flush().ok();
}

fn spinner_clear_line() {
    print!("\r\x1b[2K");
    io::stdout().flush().ok();
}

/// Run a command with a spinner showing progress
pub fn run_command_with_spinner(
    command: &str,
    args: &[&str],
    message: &str,
) -> Result<std::process::ExitStatus, String> {
    let mut child = Command::new(command)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn {}: {}", command, e))?;

    // Get stdout handle for reading output
    let stdout = child.stdout.take().unwrap();
    let current_status = Arc::new(Mutex::new(message.to_string()));

    // Start thread to read and parse output
    start_output_reader(stdout, Arc::clone(&current_status));

    // Show spinner with dynamic status updates
    let mut i = 0;
    loop {
        let current_msg = current_status.lock().unwrap().clone();
        spinner_print_frame(&current_msg, i);

        // Check if process is done
        match child.try_wait() {
            Ok(Some(status)) => {
                // Clear spinner line
                spinner_clear_line();
                return Ok(status);
            }
            Ok(None) => {
                // Still running, continue
                std::thread::sleep(Duration::from_millis(
                    crate::internal::constants::SPINNER_DELAY_MS,
                ));
                i += 1;
            }
            Err(e) => {
                spinner_clear_line();
                return Err(format!("Failed to wait for command: {}", e));
            }
        }
    }
}

/// Run a command with spinner and capture stderr for diagnostics
pub fn run_command_with_spinner_capture(
    command: &str,
    args: &[&str],
    message: &str,
) -> Result<(std::process::ExitStatus, String), String> {
    let mut child = Command::new(command)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn {}: {}", command, e))?;

    // Take stdout/stderr for reading
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let current_status = Arc::new(Mutex::new(message.to_string()));
    let captured_stderr = Arc::new(Mutex::new(String::new()));

    // Start readers
    start_output_reader(stdout, Arc::clone(&current_status));

    // Capture stderr fully for diagnostics
    {
        let captured_stderr = Arc::clone(&captured_stderr);
        thread::spawn(move || {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(stderr);
            for line in reader.lines().map_while(Result::ok) {
                let mut buf = captured_stderr.lock().unwrap();
                buf.push_str(&line);
                buf.push('\n');
            }
        });
    }

    // Show spinner with dynamic status updates
    let mut i = 0;
    let exit_status = loop {
        let current_msg = current_status.lock().unwrap().clone();
        spinner_print_frame(&current_msg, i);

        match child.try_wait() {
            Ok(Some(status)) => {
                // Clear spinner line
                spinner_clear_line();
                break status;
            }
            Ok(None) => {
                std::thread::sleep(Duration::from_millis(
                    crate::internal::constants::SPINNER_DELAY_MS,
                ));
                i += 1;
            }
            Err(e) => {
                spinner_clear_line();
                return Err(format!("Failed to wait for command: {}", e));
            }
        }
    };

    let stderr_output = captured_stderr.lock().unwrap().clone();
    Ok((exit_status, stderr_output))
}

/// Run an operation with a spinner showing progress
pub fn run_with_spinner<T, F>(operation: F, message: &str) -> Result<T, String>
where
    F: FnOnce() -> Result<T, String> + Send + 'static,
    T: Send + 'static,
{
    // Channel to communicate result from operation thread
    let (tx, rx) = std::sync::mpsc::channel();

    // Spawn thread for the operation
    thread::spawn(move || {
        let result = operation();
        let _ = tx.send(result);
    });

    // Animate spinner in main thread
    let mut i = 0;
    loop {
        spinner_print_frame(message, i);

        // Check if operation is done
        match rx.try_recv() {
            Ok(result) => {
                // Clear spinner line
                spinner_clear_line();
                return result;
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                // Operation still running, continue spinning
                thread::sleep(Duration::from_millis(
                    crate::internal::constants::SPINNER_DELAY_MS,
                ));
                i += 1;
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                // Operation thread panicked or ended unexpectedly
                spinner_clear_line();
                return Err("Operation thread ended unexpectedly".to_string());
            }
        }
    }
}

fn start_output_reader(stdout: std::process::ChildStdout, status: Arc<Mutex<String>>) {
    thread::spawn(move || {
        use std::io::{BufRead, BufReader};
        let reader = BufReader::new(stdout);

        for line in reader.lines().map_while(Result::ok) {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with("::") {
                let status_msg = if let Some(pkg) = extract_package_name(line) {
                    if line.contains("upgrading") {
                        format!("Upgrading {}", pkg)
                    } else if line.contains("installing") {
                        format!("Installing {}", pkg)
                    } else {
                        line.to_string()
                    }
                } else {
                    line.to_string()
                };
                *status.lock().unwrap() = status_msg;
            }
        }
    });
}

/// Extract package name from common paru/pacman output patterns
fn extract_package_name(line: &str) -> Option<String> {
    // Try parentheses pattern first
    if let Some(pkg_part) = line.split('(').nth(1)?.split(')').next() {
        return pkg_part.split('-').next().map(|s| s.to_string());
    }

    // Fallback for upgrading/installing lines
    if line.contains("upgrading") || line.contains("installing") {
        return line
            .split_whitespace()
            .find(|word| {
                word.contains('-') && word.chars().all(|c| c.is_alphanumeric() || c == '-')
            })
            .and_then(|word| word.split('-').next())
            .map(|s| s.to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_with_spinner() {
        let result: Result<i32, String> = run_with_spinner(|| Ok(42), "Testing spinner");
        assert_eq!(result.unwrap(), 42);
    }
}
