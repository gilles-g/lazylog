use crate::log::event::LogEvent;
use crate::log::parser::LogParser;
use crate::log::parsers::nginx_access::NginxAccessParser;

pub struct ApacheAccessParser;

impl LogParser for ApacheAccessParser {
    fn parse(&self, line_no: u32, offset: u64, len: u32, line: &str) -> Option<LogEvent> {
        let mut ev = NginxAccessParser.parse(line_no, offset, len, line)?;
        ev.source = Some("apache".into());
        Some(ev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_format() {
        let l = r#"1.2.3.4 - - [15/Jan/2024:10:23:45 +0000] "GET /x HTTP/1.1" 404 12"#;
        let ev = ApacheAccessParser.parse(1, 0, l.len() as u32, l).unwrap();
        assert_eq!(ev.source.as_deref(), Some("apache"));
        assert_eq!(ev.fields.get("status").map(String::as_str), Some("404"));
    }
}
