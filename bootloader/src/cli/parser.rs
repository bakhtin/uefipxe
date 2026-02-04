use super::commands::Command;
use crate::util::{Error, Result};
use heapless::String;

const MAX_URL_LEN: usize = 256;

/// Parse a command string into a Command
pub fn parse_command(input: &str) -> Result<Command> {
    let input = input.trim();

    if input.is_empty() {
        return Err(Error::Parse);
    }

    // Split into command and arguments
    let mut parts = input.split_whitespace();
    let cmd = parts.next().ok_or(Error::Parse)?;

    match cmd.to_lowercase().as_str() {
        "help" | "h" | "?" => Ok(Command::Help),

        "list" | "ls" => Ok(Command::List),

        "add" => {
            let url = parts.next().ok_or(Error::InvalidArgument)?;
            let mut url_string = String::new();
            url_string.push_str(url).map_err(|_| Error::BufferTooSmall)?;
            Ok(Command::Add(url_string))
        }

        "remove" | "rm" => {
            let index_str = parts.next().ok_or(Error::InvalidArgument)?;
            let index = index_str.parse::<usize>().map_err(|_| Error::Parse)?;
            Ok(Command::Remove(index))
        }

        "boot" => {
            let index_str = parts.next().ok_or(Error::InvalidArgument)?;
            let index = index_str.parse::<usize>().map_err(|_| Error::Parse)?;
            Ok(Command::Boot(index))
        }

        "default" => {
            let index_str = parts.next().ok_or(Error::InvalidArgument)?;
            let index = index_str.parse::<usize>().map_err(|_| Error::Parse)?;
            Ok(Command::Default(index))
        }

        "save" => Ok(Command::Save),

        "test-network" | "test" => Ok(Command::TestNetwork),

        "logs" => Ok(Command::Logs),

        "exit" | "quit" | "q" => Ok(Command::Exit),

        _ => Err(Error::InvalidCommand),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_help() {
        assert!(matches!(parse_command("help"), Ok(Command::Help)));
        assert!(matches!(parse_command("h"), Ok(Command::Help)));
        assert!(matches!(parse_command("?"), Ok(Command::Help)));
    }

    #[test]
    fn test_parse_list() {
        assert!(matches!(parse_command("list"), Ok(Command::List)));
        assert!(matches!(parse_command("ls"), Ok(Command::List)));
    }

    #[test]
    fn test_parse_exit() {
        assert!(matches!(parse_command("exit"), Ok(Command::Exit)));
        assert!(matches!(parse_command("quit"), Ok(Command::Exit)));
        assert!(matches!(parse_command("q"), Ok(Command::Exit)));
    }

    #[test]
    fn test_parse_add() {
        let result = parse_command("add https://example.com/image.efi");
        assert!(matches!(result, Ok(Command::Add(_))));
    }

    #[test]
    fn test_parse_remove() {
        let result = parse_command("remove 0");
        assert!(matches!(result, Ok(Command::Remove(0))));
    }
}
