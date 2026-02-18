pub mod stream;

#[derive(Clone, Debug, PartialEq)]
pub enum LogLevel {
    Default,
    Info,
    Debug,
    Error,
    Fault,
}

impl LogLevel {
    pub fn as_str(&self) -> &str {
        match self {
            LogLevel::Default => "DEFAULT",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Error => "ERROR",
            LogLevel::Fault => "FAULT",
        }
    }
}

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub process: String,
    pub message: String,
}
