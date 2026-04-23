use std::path::{Path, PathBuf};

use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct CandidateLog {
    pub path: PathBuf,
    pub size: u64,
}

pub fn scan(cwd: &Path) -> Vec<CandidateLog> {
    let mut out = Vec::new();
    // Project-local: var/log and var/logs
    for sub in ["var/log", "var/logs", "logs", "log"] {
        collect(&cwd.join(sub), 2, &mut out);
    }
    // System: /var/log (depth 1 to avoid huge walk)
    collect(Path::new("/var/log"), 1, &mut out);
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out.dedup_by(|a, b| a.path == b.path);
    out
}

fn collect(root: &Path, depth: usize, out: &mut Vec<CandidateLog>) {
    if !root.exists() {
        return;
    }
    for entry in WalkDir::new(root)
        .max_depth(depth)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let p = entry.path();
        if !is_log_like(p) {
            continue;
        }
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        out.push(CandidateLog {
            path: p.to_path_buf(),
            size,
        });
    }
}

fn is_log_like(p: &Path) -> bool {
    let raw = match p.file_name().and_then(|n| n.to_str()) {
        Some(n) => n.to_ascii_lowercase(),
        None => return false,
    };
    if raw.ends_with(".bz2") || raw.ends_with(".zip") {
        return false;
    }
    // Treat `foo.log.gz` like `foo.log` — we can read gzip-compressed logs.
    let name = raw.strip_suffix(".gz").unwrap_or(raw.as_str());
    if name.ends_with(".log") {
        return true;
    }
    name.contains(".log.") || name.ends_with("_log") || name == "syslog" || name == "messages"
}
