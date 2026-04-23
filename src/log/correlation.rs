//! Best-effort extraction of a "correlation id" from a log event.
//!
//! We look for common tracing / request-id conventions across Symfony, nginx
//! custom logs, and generic JSON contexts. The returned string is meant to be
//! fed into the full-text filter so every line sharing the same id clusters
//! together.

use once_cell::sync::Lazy;
use regex::Regex;

use crate::log::event::LogEvent;
use crate::log::source::FileSource;

/// Field keys we inspect first (cheapest path — already indexed by parsers).
const DIRECT_KEYS: &[&str] = &[
    "trace_id",
    "traceId",
    "request_id",
    "requestId",
    "correlation_id",
    "correlationId",
    "x-request-id",
    "x_request_id",
    "token",
];

/// Regex fallback. Captures the first matching id-like key/value pair in the
/// raw line (JSON-ish or bracketed). Capture group 1 is always the value.
static PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    let raw = [
        // "trace_id":"abc" or "requestId":"abc" — case-insensitive key, strict value.
        r#"(?i)"(?:trace[_-]?id|request[_-]?id|correlation[_-]?id|x[_-]?request[_-]?id|token)"\s*:\s*"([^"\s]+)""#,
        // [token] => abc   (Monolog extra array, non-JSON form)
        r#"(?i)[\[\{]\s*"?token"?\s*(?:=>|:)\s*"?([A-Za-z0-9][A-Za-z0-9_\-]{5,})"?"#,
    ];
    raw.iter().map(|p| Regex::new(p).unwrap()).collect()
});

/// Returns a correlation id for this event, if any recognizable field is present.
pub fn extract(event: &LogEvent, source: &FileSource) -> Option<String> {
    for key in DIRECT_KEYS {
        if let Some(v) = event.fields.get(*key) {
            if !v.is_empty() {
                return Some(v.clone());
            }
        }
    }
    // Symfony stores the raw JSON context under "context" / "extra". We scan
    // them first (cheap, already extracted) before falling back to the raw
    // line, which may be large.
    for field in ["context", "extra"] {
        if let Some(v) = event.fields.get(field) {
            if let Some(id) = scan(v) {
                return Some(id);
            }
        }
    }
    let raw = source.slice(event.offset, event.len);
    scan(raw)
}

fn scan(haystack: &str) -> Option<String> {
    for re in PATTERNS.iter() {
        if let Some(c) = re.captures(haystack) {
            if let Some(m) = c.get(1) {
                let v = m.as_str();
                if !v.is_empty() {
                    return Some(v.to_string());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::scan;

    #[test]
    fn scans_trace_id() {
        assert_eq!(
            scan(r#"{"trace_id":"abc-123","msg":"x"}"#).as_deref(),
            Some("abc-123")
        );
    }

    #[test]
    fn scans_request_id_variants() {
        assert_eq!(
            scan(r#"msg "requestId":"xyz789""#).as_deref(),
            Some("xyz789")
        );
        assert_eq!(
            scan(r#"[X-Request-Id] garbage "x-request-id":"xx-1""#).as_deref(),
            Some("xx-1")
        );
    }

    #[test]
    fn scans_monolog_token() {
        assert_eq!(
            scan(r#"[token => "abc12345"]"#).as_deref(),
            Some("abc12345")
        );
    }

    #[test]
    fn nothing_to_find() {
        assert!(scan("some boring line with no id").is_none());
    }
}
