use super::parser::parse_command;
use crate::util::{Error, Result};
use heapless::String;
use uefi::{println, proto::console::text::Key};
use alloc::format;
use core::time::Duration;

const MAX_INPUT_LEN: usize = 256;
const PROMPT: &str = "uefipxe> ";

/// Main REPL (Read-Eval-Print Loop)
pub fn run() -> Result<()> {
    println!();
    println!("Welcome to UEFI PXE Bootloader CLI");
    println!("Type 'help' for available commands");
    println!();

    loop {
        // Print prompt
        print_prompt();

        // Read line
        let line = match read_line() {
            Ok(line) => line,
            Err(Error::Uefi(uefi::Status::ABORTED)) => {
                // User pressed Ctrl+C or similar
                println!();
                println!("Interrupted");
                continue;
            }
            Err(e) => {
                println!("Error reading input: {:?}", e);
                continue;
            }
        };

        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        // Log the command
        crate::util::logger::log_entry(log::Level::Info, &format!("Command: {}", line));

        // Parse and execute command
        match parse_command(&line) {
            Ok(cmd) => {
                // Check if it's an exit command
                if matches!(cmd, super::commands::Command::Exit) {
                    println!("Goodbye!");
                    return Ok(());
                }

                // Execute command
                if let Err(e) = cmd.execute() {
                    println!("Error executing command: {}", e);
                    crate::util::logger::log_entry(
                        log::Level::Error,
                        &format!("Command error: {}", e),
                    );
                }
            }
            Err(Error::InvalidCommand) => {
                println!("Unknown command. Type 'help' for available commands.");
            }
            Err(Error::InvalidArgument) => {
                println!("Invalid argument. Type 'help' for usage information.");
            }
            Err(e) => {
                println!("Error parsing command: {}", e);
            }
        }
    }
}

fn print_prompt() {
    uefi::print!("{}", PROMPT);
}

/// Read a line of input from the user
fn read_line() -> Result<String<MAX_INPUT_LEN>> {
    let mut buffer = String::<MAX_INPUT_LEN>::new();

    loop {
        // Wait for key press
        let key = wait_for_key()?;

        match key {
            Key::Printable(char) => {
                // Handle printable characters
                let c: char = char.into();

                // Check for Enter key (carriage return or line feed)
                if c == '\r' || c == '\n' {
                    println!();
                    return Ok(buffer);
                }

                // Check for backspace
                if c == '\x08' || c == '\x7f' {
                    if !buffer.is_empty() {
                        buffer.pop();
                        // Move cursor back, print space, move back again
                        uefi::print!("\x08 \x08");
                    }
                    continue;
                }

                // Check if buffer is full
                if buffer.len() >= MAX_INPUT_LEN - 1 {
                    // Buffer full, beep or ignore
                    continue;
                }

                // Echo character
                uefi::print!("{}", c);

                // Add to buffer
                buffer.push(c).map_err(|_| Error::BufferTooSmall)?;
            }
            Key::Special(special) => {
                use uefi::proto::console::text::ScanCode;

                match special {
                    // Enter - return the line
                    ScanCode::NULL => {
                        println!();
                        return Ok(buffer);
                    }
                    // Backspace
                    ScanCode::DELETE => {
                        if !buffer.is_empty() {
                            buffer.pop();
                            // Move cursor back, print space, move back again
                            uefi::print!("\x08 \x08");
                        }
                    }
                    // Escape
                    ScanCode::ESCAPE => {
                        return Err(Error::Uefi(uefi::Status::ABORTED));
                    }
                    // Other special keys - ignore for now
                    _ => {}
                }
            }
        }
    }
}

/// Wait for a key press
fn wait_for_key() -> Result<Key> {
    use uefi::boot;

    loop {
        // Check if key is available by accessing stdin within the closure scope
        let key_result = uefi::system::with_stdin(|stdin| stdin.read_key());

        match key_result {
            Ok(Some(key)) => return Ok(key),
            Ok(None) => {
                // No key available, wait a bit
                boot::stall(Duration::from_micros(10_000)); // 10ms
            }
            Err(e) => return Err(Error::Uefi(e.status())),
        }
    }
}
