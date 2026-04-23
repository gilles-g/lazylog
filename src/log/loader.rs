use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use chrono::{DateTime, FixedOffset};

use crate::log::event::LogEvent;
use crate::log::format::LogFormat;
use crate::log::geoip::GeoDb;
use crate::log::parser::parser_for;
use crate::log::source::FileSource;

const CHUNK: usize = 10_000;

#[derive(Debug, Clone, Copy, Default)]
pub struct DateFilter {
    pub from: Option<DateTime<FixedOffset>>,
    pub to: Option<DateTime<FixedOffset>>,
}

impl DateFilter {
    pub fn is_empty(&self) -> bool {
        self.from.is_none() && self.to.is_none()
    }

    pub fn admits(&self, ts: Option<DateTime<FixedOffset>>) -> bool {
        match ts {
            Some(t) => {
                if let Some(f) = self.from {
                    if t < f {
                        return false;
                    }
                }
                if let Some(b) = self.to {
                    if t > b {
                        return false;
                    }
                }
                true
            }
            None => self.is_empty(),
        }
    }
}

pub enum LoadMsg {
    Source(Arc<FileSource>),
    Chunk(Vec<LogEvent>),
    Done { total: u32, cancelled: bool },
    Error(String),
}

pub struct LoadHandle {
    pub rx: mpsc::Receiver<LoadMsg>,
    cancel: Arc<AtomicBool>,
    _handle: JoinHandle<()>,
}

impl LoadHandle {
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

#[derive(Debug, Clone, Copy)]
struct LineIdx {
    line_no: u32,
    offset: u64,
    len: u32,
}

pub fn load(
    path: &Path,
    format: LogFormat,
    date_filter: DateFilter,
    geo: Option<Arc<GeoDb>>,
) -> LoadHandle {
    let path = PathBuf::from(path);
    let (tx, rx) = mpsc::channel::<LoadMsg>();
    let cancel = Arc::new(AtomicBool::new(false));
    let cancel_thread = Arc::clone(&cancel);
    let handle = thread::spawn(move || {
        let source = match FileSource::open(&path) {
            Ok(s) => s,
            Err(e) => {
                let _ = tx.send(LoadMsg::Error(format!("open {}: {e}", path.display())));
                return;
            }
        };
        if tx.send(LoadMsg::Source(Arc::clone(&source))).is_err() {
            return;
        }
        let parser = parser_for(format);
        let bytes = source.bytes();

        let index = match build_line_index(bytes, &cancel_thread) {
            Some(idx) => idx,
            None => {
                let _ = tx.send(LoadMsg::Done {
                    total: 0,
                    cancelled: true,
                });
                return;
            }
        };
        let total_lines = index.last().map(|l| l.line_no).unwrap_or(0);

        let mut buf: Vec<LogEvent> = Vec::with_capacity(CHUNK);
        // Reverse iteration: newest file lines first.
        for entry in index.iter().rev() {
            if cancel_thread.load(Ordering::Relaxed) {
                if !buf.is_empty() {
                    let mut chunk = std::mem::take(&mut buf);
                    enrich(&mut chunk, geo.as_deref());
                    let _ = tx.send(LoadMsg::Chunk(chunk));
                }
                let _ = tx.send(LoadMsg::Done {
                    total: total_lines,
                    cancelled: true,
                });
                return;
            }
            push_line(&*parser, bytes, entry, &date_filter, &mut buf);
            if buf.len() >= CHUNK {
                let mut chunk = std::mem::replace(&mut buf, Vec::with_capacity(CHUNK));
                enrich(&mut chunk, geo.as_deref());
                if tx.send(LoadMsg::Chunk(chunk)).is_err() {
                    return;
                }
            }
        }
        if !buf.is_empty() {
            enrich(&mut buf, geo.as_deref());
            let _ = tx.send(LoadMsg::Chunk(buf));
        }
        let _ = tx.send(LoadMsg::Done {
            total: total_lines,
            cancelled: false,
        });
    });
    LoadHandle {
        rx,
        cancel,
        _handle: handle,
    }
}

/// Fast forward scan to collect (line_no, offset, len) for every non-empty line.
/// Returns None if the caller cancelled mid-scan.
fn build_line_index(bytes: &[u8], cancel: &AtomicBool) -> Option<Vec<LineIdx>> {
    let mut out: Vec<LineIdx> = Vec::with_capacity(bytes.len() / 80);
    let mut line_start: usize = 0;
    let mut line_no: u32 = 0;
    let mut since_check: usize = 0;
    for nl in memchr::memchr_iter(b'\n', bytes) {
        line_no = line_no.saturating_add(1);
        let mut end = nl;
        if end > line_start && bytes[end - 1] == b'\r' {
            end -= 1;
        }
        if end > line_start {
            out.push(LineIdx {
                line_no,
                offset: line_start as u64,
                len: (end - line_start) as u32,
            });
        }
        line_start = nl + 1;
        since_check += 1;
        if since_check >= 100_000 {
            since_check = 0;
            if cancel.load(Ordering::Relaxed) {
                return None;
            }
        }
    }
    if line_start < bytes.len() {
        line_no = line_no.saturating_add(1);
        out.push(LineIdx {
            line_no,
            offset: line_start as u64,
            len: (bytes.len() - line_start) as u32,
        });
    }
    Some(out)
}

fn push_line(
    parser: &dyn crate::log::parser::LogParser,
    bytes: &[u8],
    entry: &LineIdx,
    filter: &DateFilter,
    buf: &mut Vec<LogEvent>,
) {
    let start = entry.offset as usize;
    let end = start + entry.len as usize;
    let text = std::str::from_utf8(&bytes[start..end]).unwrap_or("");
    if text.is_empty() {
        return;
    }
    let ev = parser
        .parse(entry.line_no, entry.offset, entry.len, text)
        .unwrap_or_else(|| LogEvent::unparsed(entry.line_no, entry.offset, entry.len, text));
    if !filter.admits(ev.timestamp) {
        return;
    }
    buf.push(ev);
}

pub(crate) fn enrich(chunk: &mut [LogEvent], geo: Option<&GeoDb>) {
    for ev in chunk.iter_mut() {
        let ip = match ev.fields.get("ip") {
            Some(v) => v.clone(),
            None => continue,
        };
        if !ev.fields.contains_key("subnet") {
            if let Some(s) = subnet_v4(&ip) {
                ev.fields.insert("subnet".into(), s);
            }
        }
        if let Some(geo) = geo {
            if !ev.fields.contains_key("country") {
                if let Some(country) = geo.country_label(&ip) {
                    ev.fields.insert("country".into(), country);
                }
            }
        }
    }
}

/// For an IPv4 address, returns `a.b.c.0/24`. For IPv6 or a malformed input,
/// returns `None` (keeps the facet clean — IPv6 "/64" is rarely what the user
/// wants for log triage and would add noise).
fn subnet_v4(ip: &str) -> Option<String> {
    let mut parts = ip.split('.');
    let a = parts.next()?;
    let b = parts.next()?;
    let c = parts.next()?;
    let d = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    for p in [a, b, c, d] {
        if p.is_empty() || p.len() > 3 || !p.bytes().all(|x| x.is_ascii_digit()) {
            return None;
        }
    }
    Some(format!("{a}.{b}.{c}.0/24"))
}

/// Parse (cheaply) the last timestamped line to help size default presets.
pub fn probe_last_timestamp(
    source: &FileSource,
    format: LogFormat,
) -> Option<DateTime<FixedOffset>> {
    let parser = parser_for(format);
    let bytes = source.bytes();
    let mut end = bytes.len();
    for _ in 0..2_000 {
        if end == 0 {
            break;
        }
        let search = &bytes[..end];
        let start = memchr::memrchr(b'\n', search).map(|p| p + 1).unwrap_or(0);
        let mut line_end = end;
        if line_end > start && bytes[line_end - 1] == b'\r' {
            line_end -= 1;
        }
        if line_end > start {
            if let Ok(text) = std::str::from_utf8(&bytes[start..line_end]) {
                if let Some(ev) = parser.parse(0, start as u64, (line_end - start) as u32, text) {
                    if let Some(ts) = ev.timestamp {
                        return Some(ts);
                    }
                }
            }
        }
        if start == 0 {
            break;
        }
        end = start - 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::subnet_v4;

    #[test]
    fn subnet_v4_valid() {
        assert_eq!(subnet_v4("10.0.0.1").as_deref(), Some("10.0.0.0/24"));
        assert_eq!(
            subnet_v4("192.168.42.250").as_deref(),
            Some("192.168.42.0/24")
        );
        assert_eq!(subnet_v4("1.2.3.4").as_deref(), Some("1.2.3.0/24"));
    }

    #[test]
    fn subnet_v4_rejects_non_ipv4() {
        assert!(subnet_v4("::1").is_none());
        assert!(subnet_v4("2001:db8::1").is_none());
        assert!(subnet_v4("").is_none());
        assert!(subnet_v4("10.0.0").is_none());
        assert!(subnet_v4("10.0.0.1.5").is_none());
        assert!(subnet_v4("10.0.0.a").is_none());
        assert!(subnet_v4("1234.0.0.1").is_none());
    }
}
