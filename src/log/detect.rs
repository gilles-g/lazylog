use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::log::format::LogFormat;
use crate::log::parser::parser_for;

const SAMPLE_LINES: usize = 50;
const MIN_SCORE: f32 = 0.5;

pub fn detect_from_path(path: &Path) -> LogFormat {
    if let Some(hint) = hint_from_name(path) {
        // Still verify against content to avoid false positives on misnamed files.
        if let Some(score) = score_format(path, hint) {
            if score >= MIN_SCORE {
                return hint;
            }
        }
    }
    detect_from_content(path)
}

fn hint_from_name(path: &Path) -> Option<LogFormat> {
    let raw = path.file_name()?.to_string_lossy().to_ascii_lowercase();
    // Strip a trailing `.gz` so a file named `access.log.gz` matches the same
    // rules as `access.log`.
    let name: String = raw.strip_suffix(".gz").map(str::to_string).unwrap_or(raw);
    if name.contains("php") && (name.contains("error") || name.contains("err")) {
        return Some(LogFormat::PhpError);
    }
    if name.contains("nginx") && name.contains("access") {
        return Some(LogFormat::NginxAccess);
    }
    if name.contains("nginx") && name.contains("error") {
        return Some(LogFormat::NginxError);
    }
    if name.contains("apache") || name.contains("httpd") {
        if name.contains("access") {
            return Some(LogFormat::ApacheAccess);
        }
        if name.contains("error") {
            return Some(LogFormat::ApacheError);
        }
    }
    if name.ends_with("prod.log") || name.ends_with("dev.log") || name.contains("symfony") {
        return Some(LogFormat::SymfonyMonolog);
    }
    if name == "access.log" || name.starts_with("access.log.") {
        return Some(LogFormat::NginxAccess);
    }
    if name == "error.log" || name.starts_with("error.log.") {
        return Some(LogFormat::NginxError);
    }
    None
}

fn detect_from_content(path: &Path) -> LogFormat {
    let mut best = (LogFormat::Generic, 0.0_f32);
    for fmt in LogFormat::all() {
        if matches!(fmt, LogFormat::Generic) {
            continue;
        }
        if let Some(score) = score_format(path, *fmt) {
            if score > best.1 {
                best = (*fmt, score);
            }
        }
    }
    if best.1 >= MIN_SCORE {
        best.0
    } else {
        LogFormat::Generic
    }
}

fn score_format(path: &Path, format: LogFormat) -> Option<f32> {
    let reader = open_reader(path)?;
    let parser = parser_for(format);
    let mut total = 0_usize;
    let mut hits = 0_usize;
    for line in reader.lines().map_while(Result::ok) {
        if line.trim().is_empty() {
            continue;
        }
        total += 1;
        if parser
            .parse(total as u32, 0, line.len() as u32, &line)
            .is_some()
        {
            hits += 1;
        }
        if total >= SAMPLE_LINES {
            break;
        }
    }
    if total == 0 {
        return None;
    }
    Some(hits as f32 / total as f32)
}

fn open_reader(path: &Path) -> Option<Box<dyn BufRead>> {
    let file = std::fs::File::open(path).ok()?;
    if is_gzip(path) {
        Some(Box::new(BufReader::new(flate2::read::MultiGzDecoder::new(
            file,
        ))))
    } else {
        Some(Box::new(BufReader::new(file)))
    }
}

fn is_gzip(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("gz"))
        .unwrap_or(false)
}
