use heapless::Deque;
use core::fmt::Write;

const LOG_BUFFER_SIZE: usize = 100;
const MAX_LOG_ENTRY_LEN: usize = 128;

/// A single log entry
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub level: log::Level,
    pub message: heapless::String<MAX_LOG_ENTRY_LEN>,
}

/// Global log buffer using a circular buffer
static mut LOG_BUFFER: Option<Deque<LogEntry, LOG_BUFFER_SIZE>> = None;

/// Initialize the log buffer
pub fn init() {
    unsafe {
        LOG_BUFFER = Some(Deque::new());
    }
}

/// Add a log entry to the buffer
pub fn log_entry(level: log::Level, message: &str) {
    unsafe {
        if let Some(ref mut buffer) = LOG_BUFFER {
            let mut entry = LogEntry {
                level,
                message: heapless::String::new(),
            };

            // Truncate message if too long
            let truncated = if message.len() > MAX_LOG_ENTRY_LEN - 4 {
                let mut s = heapless::String::new();
                let _ = write!(s, "{}...", &message[..MAX_LOG_ENTRY_LEN - 7]);
                s
            } else {
                let mut s = heapless::String::new();
                let _ = write!(s, "{}", message);
                s
            };

            entry.message = truncated;

            // If buffer is full, remove oldest entry
            if buffer.is_full() {
                buffer.pop_front();
            }

            let _ = buffer.push_back(entry);
        }
    }
}

/// Get all log entries
pub fn get_logs() -> heapless::Vec<LogEntry, LOG_BUFFER_SIZE> {
    unsafe {
        if let Some(ref buffer) = LOG_BUFFER {
            buffer.iter().cloned().collect()
        } else {
            heapless::Vec::new()
        }
    }
}

/// Clear all log entries
pub fn clear_logs() {
    unsafe {
        if let Some(ref mut buffer) = LOG_BUFFER {
            buffer.clear();
        }
    }
}
