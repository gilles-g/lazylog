use chrono::DateTime;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::log::event::{Level, LogEvent};
use crate::log::parser::LogParser;

static ISO_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?P<ts>\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+\-]\d{2}:?\d{2})?)\s+(?P<rest>.*)$").unwrap()
});

static LEVEL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(DEBUG|INFO|NOTICE|WARN(?:ING)?|ERR(?:OR)?|CRIT(?:ICAL)?|FATAL|ALERT|EMERG(?:ENCY)?)\b").unwrap()
});

pub struct GenericParser;

impl LogParser for GenericParser {
    fn parse(&self, line_no: u32, offset: u64, len: u32, line: &str) -> Option<LogEvent> {
        if line.trim().is_empty() {
            return None;
        }
        let mut ev = LogEvent::unparsed(line_no, offset, len, line);
        if let Some(caps) = ISO_RE.captures(line) {
            let ts_raw = caps.name("ts")?.as_str();
            ev.timestamp = parse_any(ts_raw);
            ev.message = caps.name("rest")?.as_str().to_string();
        }
        if let Some(c) = LEVEL_RE.captures(&ev.message) {
            ev.level = Level::from_str_ci(c.get(1).unwrap().as_str());
        }
        Some(ev)
    }
}

fn parse_any(s: &str) -> Option<chrono::DateTime<chrono::FixedOffset>> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt);
    }
    for fmt in &[
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S",
    ] {
        if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(s, fmt) {
            return Some(DateTime::from_naive_utc_and_offset(
                naive,
                chrono::FixedOffset::east_opt(0).unwrap(),
            ));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_up_level_and_ts() {
        let l = "2024-01-15T10:23:45Z something ERROR happened";
        let ev = GenericParser.parse(1, 0, l.len() as u32, l).unwrap();
        assert!(ev.timestamp.is_some());
        assert_eq!(ev.level, Level::Error);
    }

    #[test]
    fn fallback_still_emits_event() {
        let ev = GenericParser.parse(1, 0, 17, "no timestamp here").unwrap();
        assert_eq!(ev.message, "no timestamp here");
    }
}
