use std::collections::HashMap;
use std::thread;

use crate::log::event::LogEvent;
use crate::log::format::LogFormat;
use crate::query::filter::event_field;

#[derive(Debug, Clone)]
pub struct FacetGroup {
    pub key: String,
    pub label: String,
    pub values: Vec<(String, usize)>,
    /// Distinct value count before truncation — used to display "(+N more)".
    pub distinct: usize,
}

#[derive(Debug, Clone, Default)]
pub struct FacetIndex {
    pub groups: Vec<FacetGroup>,
}

#[derive(Debug, Clone, Copy)]
struct GroupSpec {
    key: &'static str,
    label: &'static str,
    top: usize,
}

const TOP_LO: usize = 50;
const TOP_HI: usize = 15;

fn groups_for(format: LogFormat) -> Vec<GroupSpec> {
    match format {
        LogFormat::SymfonyMonolog => vec![
            lo("level", "Level"),
            lo("channel", "Channel"),
            hi("exception", "Exception"),
        ],
        LogFormat::PhpError => vec![lo("type", "Type"), hi("file", "File")],
        LogFormat::NginxAccess | LogFormat::ApacheAccess => vec![
            lo("status_class", "Status class"),
            lo("status", "Status"),
            lo("method", "Method"),
            hi("vhost", "Vhost"),
            hi("country", "Country"),
            hi("subnet", "Subnet /24"),
            hi("ip", "Client IP"),
        ],
        LogFormat::NginxError => vec![
            lo("level", "Level"),
            lo("process", "Process"),
            hi("country", "Country"),
            hi("subnet", "Subnet /24"),
            hi("ip", "Client IP"),
        ],
        LogFormat::ApacheError => vec![
            lo("level", "Level"),
            lo("module", "Module"),
            hi("country", "Country"),
            hi("subnet", "Subnet /24"),
            hi("ip", "Client IP"),
        ],
        LogFormat::Generic => vec![lo("level", "Level")],
    }
}

const fn lo(key: &'static str, label: &'static str) -> GroupSpec {
    GroupSpec {
        key,
        label,
        top: TOP_LO,
    }
}

const fn hi(key: &'static str, label: &'static str) -> GroupSpec {
    GroupSpec {
        key,
        label,
        top: TOP_HI,
    }
}

pub fn compute(format: LogFormat, events: &[LogEvent], visible: &[u32]) -> FacetIndex {
    let specs = groups_for(format);
    // Each facet group is computed on its own thread — the slowest (typically
    // the high-cardinality Country / IP groups) no longer blocks the others.
    let groups: Vec<Option<FacetGroup>> = thread::scope(|s| {
        let handles: Vec<_> = specs
            .iter()
            .map(|spec| s.spawn(move || compute_one(*spec, events, visible)))
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().unwrap_or(None))
            .collect()
    });
    FacetIndex {
        groups: groups.into_iter().flatten().collect(),
    }
}

fn compute_one(spec: GroupSpec, events: &[LogEvent], visible: &[u32]) -> Option<FacetGroup> {
    let mut counts: HashMap<&str, usize> = HashMap::with_capacity(64);
    for &idx in visible {
        let ev = &events[idx as usize];
        if let Some(v) = event_field(ev, spec.key) {
            if v.is_empty() {
                continue;
            }
            *counts.entry(v).or_insert(0) += 1;
        }
    }
    let distinct = counts.len();
    // Skip groups that never matched any event (e.g. the Country facet
    // on an access log opened without a GeoIP database).
    if distinct == 0 {
        return None;
    }
    let mut values: Vec<(String, usize)> = counts
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();
    values.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    values.truncate(spec.top);
    Some(FacetGroup {
        key: spec.key.to_string(),
        label: spec.label.to_string(),
        values,
        distinct,
    })
}
