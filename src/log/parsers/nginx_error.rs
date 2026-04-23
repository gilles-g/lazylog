use chrono::DateTime;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::log::event::{Level, LogEvent};
use crate::log::parser::LogParser;

static RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^(?P<ts>\d{4}/\d{2}/\d{2}\s+\d{2}:\d{2}:\d{2})\s+\[(?P<level>\w+)\]\s+(?P<pid>\d+)#(?P<tid>\d+):\s+(?P<msg>.*)$",
    )
    .unwrap()
});

static CLIENT_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"client:\s*(?P<ip>[0-9a-fA-F\.:]+)").unwrap());

pub struct NginxErrorParser;

impl LogParser for NginxErrorParser {
    fn parse(&self, line_no: u32, offset: u64, len: u32, line: &str) -> Option<LogEvent> {
        let caps = RE.captures(line)?;
        let ts = caps.name("ts")?.as_str();
        let level_s = caps.name("level")?.as_str();
        let level = Level::from_str_ci(level_s);
        let pid = caps.name("pid")?.as_str().to_string();
        let msg = caps.name("msg")?.as_str().trim().to_string();

        let mut ev = LogEvent::unparsed(line_no, offset, len, "");
        ev.timestamp = parse_ts(ts);
        ev.level = level;
        ev.source = Some("nginx".into());
        ev.message = msg.clone();
        ev.fields.insert("process".into(), pid);
        if let Some(c) = CLIENT_RE.captures(&msg) {
            if let Some(ip) = c.name("ip") {
                ev.fields.insert("ip".into(), ip.as_str().into());
            }
        }
        Some(ev)
    }
}

fn parse_ts(s: &str) -> Option<chrono::DateTime<chrono::FixedOffset>> {
    let naive = chrono::NaiveDateTime::parse_from_str(s, "%Y/%m/%d %H:%M:%S").ok()?;
    Some(DateTime::from_naive_utc_and_offset(
        naive,
        chrono::FixedOffset::east_opt(0).unwrap(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_error_line() {
        let l = "2024/01/15 10:23:45 [error] 1234#5678: *42 open() failed, client: 1.2.3.4, server: foo";
        let ev = NginxErrorParser.parse(1, 0, l.len() as u32, l).unwrap();
        assert_eq!(ev.level, Level::Error);
        assert_eq!(ev.fields.get("ip").map(String::as_str), Some("1.2.3.4"));
        assert_eq!(ev.fields.get("process").map(String::as_str), Some("1234"));
    }
}
