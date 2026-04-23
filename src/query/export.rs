use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Local;

use crate::log::event::LogEvent;
use crate::log::source::FileSource;
use crate::query::filter::event_field;

/// Recompute the full (untruncated) value → count map for `group_key` over the
/// currently visible events, then write it to a `.txt` file next to the log
/// source. Returns the written path.
pub fn export_group(
    log_path: &Path,
    group_key: &str,
    events: &[LogEvent],
    visible: &[u32],
) -> Result<PathBuf> {
    let mut counts: HashMap<&str, usize> = HashMap::with_capacity(256);
    for &idx in visible {
        let ev = &events[idx as usize];
        if let Some(v) = event_field(ev, group_key) {
            if v.is_empty() {
                continue;
            }
            *counts.entry(v).or_insert(0) += 1;
        }
    }
    let mut rows: Vec<(&str, usize)> = counts.into_iter().collect();
    rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));

    let dir = log_path.parent().unwrap_or_else(|| Path::new("."));
    let stem = log_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("log");
    let ts = Local::now().format("%Y%m%d-%H%M%S");
    let safe_key = sanitize_key(group_key);
    let out = dir.join(format!("lazylog-{stem}-{safe_key}-{ts}.txt"));

    let file = File::create(&out).with_context(|| format!("create {}", out.display()))?;
    let mut w = BufWriter::new(file);
    for (value, count) in &rows {
        writeln!(w, "{value}\t{count}")?;
    }
    w.flush()?;
    Ok(out)
}

/// Writes every visible log line (raw source bytes, in display order —
/// oldest first, same as the Events panel) to a `.txt` next to the source.
pub fn export_filtered(
    log_path: &Path,
    events: &[LogEvent],
    visible: &[u32],
    source: &FileSource,
) -> Result<PathBuf> {
    let dir = log_path.parent().unwrap_or_else(|| Path::new("."));
    let stem = log_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("log");
    let ts = Local::now().format("%Y%m%d-%H%M%S");
    let out = dir.join(format!("lazylog-{stem}-filtered-{ts}.txt"));

    let file = File::create(&out).with_context(|| format!("create {}", out.display()))?;
    let mut w = BufWriter::new(file);
    for &idx in visible {
        let ev = &events[idx as usize];
        let line = source.slice(ev.offset, ev.len);
        writeln!(w, "{line}")?;
    }
    w.flush()?;
    Ok(out)
}

/// Keep only ASCII alphanumerics, `-` and `_` — anything else (including
/// path separators, `..`, NUL, spaces) becomes `_`. Prevents path traversal
/// if `group_key` ever becomes caller-controlled.
fn sanitize_key(key: &str) -> String {
    let mut out: String = key
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if out.is_empty() {
        out.push_str("group");
    }
    out.truncate(64);
    out
}

#[cfg(test)]
mod tests {
    use super::sanitize_key;

    #[test]
    fn sanitize_strips_path_traversal() {
        assert_eq!(sanitize_key("../../etc/passwd"), "______etc_passwd");
        assert_eq!(sanitize_key("country"), "country");
        assert_eq!(sanitize_key(""), "group");
        assert_eq!(sanitize_key("a/b\\c:d"), "a_b_c_d");
    }
}
