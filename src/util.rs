use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Run a command with a spinner showing progress
pub fn run_command_with_spinner(
    command: &str,
    args: &[&str],
    message: &str,
) -> Result<std::process::ExitStatus, String> {
    let spinner_chars = ["⁚", "⁖", "⁘", "⁛", "⁙", "⁛", "⁘", "⁖"];

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
        print!("\r\x1b[2K  {} {}...", crate::colo::blue(spinner_chars[i % spinner_chars.len()]), current_msg);
        io::stdout().flush().unwrap();

        // Check if process is done
        match child.try_wait() {
            Ok(Some(status)) => {
                // Clear spinner line
                print!("\r\x1b[2K");
                io::stdout().flush().unwrap();
                return Ok(status);
            }
            Ok(None) => {
                // Still running, continue
                std::thread::sleep(Duration::from_millis(crate::constants::SPINNER_DELAY_MS));
                i += 1;
            }
            Err(e) => {
                print!("\r\x1b[2K");
                io::stdout().flush().unwrap();
                return Err(format!("Failed to wait for command: {}", e));
            }
        }
    }
}

/// Run an operation with a spinner showing progress
pub fn run_with_spinner<T, F>(operation: F, message: &str) -> Result<T, String>
where
    F: FnOnce() -> Result<T, String> + Send + 'static,
    T: Send + 'static,
{
    let spinner_chars = ["⁚", "⁖", "⁘", "⁛", "⁙", "⁛", "⁘", "⁖"];

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
        print!("\r\x1b[2K  {} {}...", crate::colo::blue(spinner_chars[i % spinner_chars.len()]), message);
        io::stdout().flush().unwrap();

        // Check if operation is done
        match rx.try_recv() {
            Ok(result) => {
                // Clear spinner line
                print!("\r\x1b[2K");
                io::stdout().flush().unwrap();
                return result;
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                // Operation still running, continue spinning
                thread::sleep(Duration::from_millis(crate::constants::SPINNER_DELAY_MS));
                i += 1;
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                // Operation thread panicked or ended unexpectedly
                print!("\r\x1b[2K");
                io::stdout().flush().unwrap();
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
                let status_msg = if let Some(pkg) = extract_package_name(&line) {
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

fn show_spinner(current_status: &Arc<Mutex<String>>, spinner_chars: &[&str]) {
    let mut i = 0;
    let initial_msg = current_status.lock().unwrap().clone();
    print!("  {} {}...", crate::colo::blue(spinner_chars[0]), initial_msg);
    io::stdout().flush().unwrap();

    // Simple spinner animation - in a real implementation, this would check the actual process
    for _ in 0..10 {
        std::thread::sleep(Duration::from_millis(crate::constants::SPINNER_DELAY_MS));
        let current_msg = current_status.lock().unwrap().clone();
        print!("\r\x1b[2K  {} {}...", crate::colo::blue(spinner_chars[i % spinner_chars.len()]), current_msg);
        io::stdout().flush().unwrap();
        i += 1;
    }
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
            .find(|word| word.contains('-') && word.chars().all(|c| c.is_alphanumeric() || c == '-'))
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