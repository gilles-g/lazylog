use chrono::DateTime;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::log::event::{Level, LogEvent};
use crate::log::parser::LogParser;

// Supports both combined and vhost_combined:
//   [vhost ]IP ident user [ts] "METHOD path HTTP/x" status size ["ref" "ua"]
// The IP must look like an IPv4/IPv6 so we don't pick up a vhost by accident.
static RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"^(?:(?P<vhost>\S+)\s+)?(?P<ip>\d{1,3}(?:\.\d{1,3}){3}|[0-9a-fA-F:]+)\s+\S+\s+(?P<user>\S+)\s+\[(?P<ts>[^\]]+)\]\s+"(?P<method>[A-Z]+)\s+(?P<path>\S+)\s+(?P<proto>[^"]+)"\s+(?P<status>\d{3})\s+(?P<size>\S+)(?:\s+"(?P<ref>[^"]*)"\s+"(?P<ua>[^"]*)")?"#,
    )
    .unwrap()
});

pub struct NginxAccessParser;

impl LogParser for NginxAccessParser {
    fn parse(&self, line_no: u32, offset: u64, len: u32, line: &str) -> Option<LogEvent> {
        let caps = RE.captures(line)?;
        let status = caps.name("status")?.as_str();
        let status_n: u16 = status.parse().ok()?;
        let level = match status_n {
            500..=599 => Level::Error,
            400..=499 => Level::Warning,
            _ => Level::Info,
        };
        let method = caps.name("method")?.as_str().to_string();
        let path = caps.name("path")?.as_str().to_string();
        let ip = caps.name("ip")?.as_str().to_string();
        let ts = caps.name("ts")?.as_str();

        let mut ev = LogEvent::unparsed(line_no, offset, len, "");
        ev.timestamp = parse_clf_ts(ts);
        ev.level = level;
        ev.source = Some("nginx".into());
        ev.message = format!("{method} {path} -> {status_n}");
        ev.fields.insert("ip".into(), ip);
        ev.fields.insert("method".into(), method);
        ev.fields.insert("path".into(), path);
        ev.fields.insert("status".into(), status.into());
        ev.fields
            .insert("status_class".into(), format!("{}xx", status_n / 100));
        if let Some(vhost) = caps.name("vhost") {
            ev.fields.insert("vhost".into(), vhost.as_str().into());
        }
        if let Some(size) = caps.name("size") {
            ev.fields.insert("size".into(), size.as_str().into());
        }
        Some(ev)
    }
}

pub(crate) fn parse_clf_ts(s: &str) -> Option<chrono::DateTime<chrono::FixedOffset>> {
    DateTime::parse_from_str(s, "%d/%b/%Y:%H:%M:%S %z").ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_combined() {
        let l =
            r#"1.2.3.4 - - [15/Jan/2024:10:23:45 +0000] "GET /foo HTTP/1.1" 200 1234 "-" "curl/8""#;
        let ev = NginxAccessParser.parse(1, 0, l.len() as u32, l).unwrap();
        assert_eq!(ev.fields.get("method").map(String::as_str), Some("GET"));
        assert_eq!(ev.fields.get("status").map(String::as_str), Some("200"));
        assert_eq!(
            ev.fields.get("status_class").map(String::as_str),
            Some("2xx")
        );
        assert_eq!(ev.level, Level::Info);
    }

    #[test]
    fn flags_5xx_as_error() {
        let l = r#"1.2.3.4 - - [15/Jan/2024:10:23:45 +0000] "GET /x HTTP/1.1" 502 0 "-" "-""#;
        let ev = NginxAccessParser.parse(1, 0, l.len() as u32, l).unwrap();
        assert_eq!(ev.level, Level::Error);
    }

    #[test]
    fn parses_vhost_combined() {
        let l = r#"www.example.com 1.2.3.4 - - [15/Jan/2024:10:23:45 +0000] "GET /foo HTTP/1.1" 200 1234 "-" "curl/8""#;
        let ev = NginxAccessParser.parse(1, 0, l.len() as u32, l).unwrap();
        assert_eq!(
            ev.fields.get("vhost").map(String::as_str),
            Some("www.example.com")
        );
        assert_eq!(ev.fields.get("ip").map(String::as_str), Some("1.2.3.4"));
    }
}
