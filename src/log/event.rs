use chrono::{DateTime, FixedOffset};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Level {
    Debug,
    Info,
    Notice,
    Warning,
    Error,
    Critical,
    Alert,
    Emergency,
    Unknown,
}

impl Level {
    pub fn as_str(&self) -> &'static str {
        match self {
            Level::Debug => "DEBUG",
            Level::Info => "INFO",
            Level::Notice => "NOTICE",
            Level::Warning => "WARNING",
            Level::Error => "ERROR",
            Level::Critical => "CRITICAL",
            Level::Alert => "ALERT",
            Level::Emergency => "EMERGENCY",
            Level::Unknown => "UNKNOWN",
        }
    }

    pub fn from_str_ci(s: &str) -> Level {
        match s.to_ascii_uppercase().as_str() {
            "DEBUG" => Level::Debug,
            "INFO" | "INFORMATIONAL" => Level::Info,
            "NOTICE" => Level::Notice,
            "WARN" | "WARNING" => Level::Warning,
            "ERR" | "ERROR" => Level::Error,
            "CRIT" | "CRITICAL" | "FATAL" => Level::Critical,
            "ALERT" => Level::Alert,
            "EMERG" | "EMERGENCY" => Level::Emergency,
            _ => Level::Unknown,
        }
    }
}

/// Compact event record. The line text is not stored — it is kept in the
/// mmap-backed `FileSource` and retrieved on demand via `offset` / `len`.
#[derive(Debug, Clone)]
pub struct LogEvent {
    pub line_no: u32,
    pub offset: u64,
    pub len: u32,
    pub timestamp: Option<DateTime<FixedOffset>>,
    pub level: Level,
    pub source: Option<String>,
    pub message: String,
    pub fields: BTreeMap<String, String>,
}

impl LogEvent {
    pub fn unparsed(line_no: u32, offset: u64, len: u32, text: &str) -> Self {
        Self {
            line_no,
            offset,
            len,
            timestamp: None,
            level: Level::Unknown,
            source: None,
            message: text.to_string(),
            fields: BTreeMap::new(),
        }
    }
}
