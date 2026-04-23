use chrono::DateTime;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::log::event::{Level, LogEvent};
use crate::log::parser::LogParser;

static RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^\[(?P<ts>[^\]]+)\]\s+\[(?P<module>[\w\-]+):(?P<level>[\w]+)\]\s+\[pid\s+(?P<pid>\d+)(?::tid\s+\d+)?\](?:\s+\[client\s+(?P<client>[^\]]+)\])?\s+(?P<msg>.*)$",
    )
    .unwrap()
});

pub struct ApacheErrorParser;

impl LogParser for ApacheErrorParser {
    fn parse(&self, line_no: u32, offset: u64, len: u32, line: &str) -> Option<LogEvent> {
        let caps = RE.captures(line)?;
        let ts = caps.name("ts")?.as_str();
        let module = caps.name("module")?.as_str().to_string();
        let level = Level::from_str_ci(caps.name("level")?.as_str());
        let pid = caps.name("pid")?.as_str().to_string();
        let msg = caps.name("msg")?.as_str().trim().to_string();

        let mut ev = LogEvent::unparsed(line_no, offset, len, "");
        ev.timestamp = parse_ts(ts);
        ev.level = level;
        ev.source = Some("apache".into());
        ev.message = msg;
        ev.fields.insert("module".into(), module);
        ev.fields.insert("process".into(), pid);
        if let Some(client) = caps.name("client") {
            let c = client.as_str();
            let ip = c.split(':').next().unwrap_or(c).to_string();
            ev.fields.insert("ip".into(), ip);
        }
        Some(ev)
    }
}

fn parse_ts(s: &str) -> Option<chrono::DateTime<chrono::FixedOffset>> {
    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(s, "%a %b %d %H:%M:%S%.f %Y") {
        return Some(DateTime::from_naive_utc_and_offset(
            naive,
            chrono::FixedOffset::east_opt(0).unwrap(),
        ));
    }
    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(s, "%a %b %d %H:%M:%S %Y") {
        return Some(DateTime::from_naive_utc_and_offset(
            naive,
            chrono::FixedOffset::east_opt(0).unwrap(),
        ));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_apache_error() {
        let l = "[Mon Jan 15 10:23:45.123456 2024] [core:error] [pid 1234] [client 1.2.3.4:5678] something bad";
        let ev = ApacheErrorParser.parse(1, 0, l.len() as u32, l).unwrap();
        assert_eq!(ev.level, Level::Error);
        assert_eq!(ev.fields.get("module").map(String::as_str), Some("core"));
        assert_eq!(ev.fields.get("ip").map(String::as_str), Some("1.2.3.4"));
    }
}
