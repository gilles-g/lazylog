use chrono::DateTime;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::log::event::{Level, LogEvent};
use crate::log::parser::LogParser;

static RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^\[(?P<ts>[^\]]+)\]\s+(?P<channel>[\w\-\.]+)\.(?P<level>[A-Z]+):\s+(?P<msg>.*?)(?:\s+(?P<ctx>\{.*\}))?(?:\s+(?P<extra>\[.*\]))?\s*$",
    )
    .unwrap()
});

static EXC_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#""exception":"\[object\] \(([A-Za-z0-9_\\]+)"#).unwrap());

pub struct SymfonyParser;

impl LogParser for SymfonyParser {
    fn parse(&self, line_no: u32, offset: u64, len: u32, line: &str) -> Option<LogEvent> {
        let caps = RE.captures(line)?;
        let ts_str = caps.name("ts")?.as_str();
        let channel = caps.name("channel")?.as_str().to_string();
        let level = Level::from_str_ci(caps.name("level")?.as_str());
        let msg = caps.name("msg")?.as_str().trim().to_string();

        let mut event = LogEvent::unparsed(line_no, offset, len, "");
        event.timestamp = parse_ts(ts_str);
        event.level = level;
        event.source = Some(channel.clone());
        event.message = msg;
        event.fields.insert("channel".into(), channel);

        if let Some(ctx) = caps.name("ctx") {
            event.fields.insert("context".into(), ctx.as_str().into());
            if let Some(cls) = extract_exception_class(ctx.as_str()) {
                event.fields.insert("exception".into(), cls);
            }
        }
        if let Some(extra) = caps.name("extra") {
            event
                .fields
                .insert("extra".into(), extra.as_str().to_string());
        }
        Some(event)
    }
}

fn parse_ts(s: &str) -> Option<chrono::DateTime<chrono::FixedOffset>> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt);
    }
    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Some(DateTime::from_naive_utc_and_offset(
            naive,
            chrono::FixedOffset::east_opt(0).unwrap(),
        ));
    }
    None
}

fn extract_exception_class(ctx: &str) -> Option<String> {
    EXC_RE
        .captures(ctx)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_line() {
        let line = "[2024-01-15T10:23:45+00:00] app.ERROR: Something broke {\"user\":42} []";
        let ev = SymfonyParser.parse(1, 0, line.len() as u32, line).unwrap();
        assert_eq!(ev.level, Level::Error);
        assert_eq!(ev.source.as_deref(), Some("app"));
        assert_eq!(ev.message, "Something broke");
        assert!(ev.timestamp.is_some());
    }

    #[test]
    fn parses_exception_class() {
        let line = "[2024-01-15T10:23:45+00:00] app.CRITICAL: boom {\"exception\":\"[object] (RuntimeException(code: 0): message at /path:42)\"} []";
        let ev = SymfonyParser.parse(1, 0, line.len() as u32, line).unwrap();
        assert_eq!(
            ev.fields.get("exception").map(String::as_str),
            Some("RuntimeException")
        );
    }

    #[test]
    fn rejects_non_matching() {
        assert!(SymfonyParser.parse(1, 0, 12, "garbage line").is_none());
    }
}
