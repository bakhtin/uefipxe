use core::fmt;

/// Main error type for the bootloader
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// UEFI error
    Uefi(uefi::Status),
    /// Input/output error
    Io,
    /// Parse error
    Parse,
    /// Invalid command
    InvalidCommand,
    /// Invalid argument
    InvalidArgument,
    /// Not found
    NotFound,
    /// Out of memory
    OutOfMemory,
    /// Buffer too small
    BufferTooSmall,
    /// Unknown error
    Unknown,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Uefi(status) => write!(f, "UEFI error: {:?}", status),
            Error::Io => write!(f, "I/O error"),
            Error::Parse => write!(f, "Parse error"),
            Error::InvalidCommand => write!(f, "Invalid command"),
            Error::InvalidArgument => write!(f, "Invalid argument"),
            Error::NotFound => write!(f, "Not found"),
            Error::OutOfMemory => write!(f, "Out of memory"),
            Error::BufferTooSmall => write!(f, "Buffer too small"),
            Error::Unknown => write!(f, "Unknown error"),
        }
    }
}

impl From<uefi::Status> for Error {
    fn from(status: uefi::Status) -> Self {
        Error::Uefi(status)
    }
}

pub type Result<T> = core::result::Result<T, Error>;
