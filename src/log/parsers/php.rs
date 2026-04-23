use chrono::DateTime;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::log::event::{Level, LogEvent};
use crate::log::parser::LogParser;

static RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^\[(?P<ts>[^\]]+)\]\s+PHP\s+(?P<kind>Fatal error|Parse error|Warning|Notice|Deprecated|Strict Standards|Error):\s+(?P<msg>.+?)(?:\s+in\s+(?P<file>.+?)\s+on\s+line\s+(?P<line>\d+))?\s*$",
    )
    .unwrap()
});

pub struct PhpErrorParser;

impl LogParser for PhpErrorParser {
    fn parse(&self, line_no: u32, offset: u64, len: u32, line: &str) -> Option<LogEvent> {
        let caps = RE.captures(line)?;
        let ts_str = caps.name("ts")?.as_str();
        let kind = caps.name("kind")?.as_str().to_string();
        let msg = caps.name("msg")?.as_str().trim().to_string();

        let level = match kind.as_str() {
            "Fatal error" | "Parse error" | "Error" => Level::Critical,
            "Warning" => Level::Warning,
            "Notice" | "Deprecated" | "Strict Standards" => Level::Notice,
            _ => Level::Unknown,
        };

        let mut event = LogEvent::unparsed(line_no, offset, len, "");
        event.timestamp = parse_php_ts(ts_str);
        event.level = level;
        event.source = Some("php".into());
        event.message = msg;
        event.fields.insert("type".into(), kind);
        if let Some(file) = caps.name("file") {
            event
                .fields
                .insert("file".into(), file.as_str().to_string());
        }
        if let Some(line_m) = caps.name("line") {
            event
                .fields
                .insert("line".into(), line_m.as_str().to_string());
        }
        Some(event)
    }
}

fn parse_php_ts(s: &str) -> Option<chrono::DateTime<chrono::FixedOffset>> {
    let tz0 = chrono::FixedOffset::east_opt(0).unwrap();
    let stripped = s.trim();
    let base = stripped
        .rsplit_once(' ')
        .map(|(a, _)| a)
        .unwrap_or(stripped);
    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(base, "%d-%b-%Y %H:%M:%S") {
        return Some(DateTime::from_naive_utc_and_offset(naive, tz0));
    }
    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(stripped, "%d-%b-%Y %H:%M:%S") {
        return Some(DateTime::from_naive_utc_and_offset(naive, tz0));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_warning() {
        let l = "[15-Jan-2024 10:23:45 UTC] PHP Warning:  Undefined variable $foo in /var/www/a.php on line 12";
        let ev = PhpErrorParser.parse(1, 0, l.len() as u32, l).unwrap();
        assert_eq!(ev.level, Level::Warning);
        assert_eq!(ev.fields.get("type").map(String::as_str), Some("Warning"));
        assert_eq!(
            ev.fields.get("file").map(String::as_str),
            Some("/var/www/a.php")
        );
        assert_eq!(ev.fields.get("line").map(String::as_str), Some("12"));
    }

    #[test]
    fn parses_fatal() {
        let l = "[15-Jan-2024 10:23:45 UTC] PHP Fatal error:  Uncaught Error: foo";
        let ev = PhpErrorParser.parse(1, 0, l.len() as u32, l).unwrap();
        assert_eq!(ev.level, Level::Critical);
    }
}
