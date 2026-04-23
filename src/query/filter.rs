use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, FixedOffset};

use crate::log::event::LogEvent;
use crate::log::source::FileSource;

#[derive(Debug, Clone, Default)]
pub struct Filter {
    pub text: String,
    pub from: Option<DateTime<FixedOffset>>,
    pub to: Option<DateTime<FixedOffset>>,
    pub selections: BTreeMap<String, BTreeSet<String>>,
}

impl Filter {
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
            && self.from.is_none()
            && self.to.is_none()
            && self.selections.values().all(|s| s.is_empty())
    }

    pub fn matches(&self, ev: &LogEvent, source: &FileSource) -> bool {
        if !self.text.is_empty() {
            let raw = source.slice(ev.offset, ev.len);
            if !contains_ci(raw, &self.text) {
                return false;
            }
        }
        if let Some(ts) = ev.timestamp {
            if let Some(from) = self.from {
                if ts < from {
                    return false;
                }
            }
            if let Some(to) = self.to {
                if ts > to {
                    return false;
                }
            }
        } else if self.from.is_some() || self.to.is_some() {
            return false;
        }
        for (group, allowed) in &self.selections {
            if allowed.is_empty() {
                continue;
            }
            let actual = event_field(ev, group);
            match actual {
                Some(v) if allowed.contains(v) => {}
                _ => return false,
            }
        }
        true
    }

    pub fn toggle(&mut self, group: &str, value: &str) {
        let set = self.selections.entry(group.to_string()).or_default();
        if !set.insert(value.to_string()) {
            set.remove(value);
        }
    }

    pub fn clear_group(&mut self, group: &str) {
        if let Some(s) = self.selections.get_mut(group) {
            s.clear();
        }
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

pub fn event_field<'a>(ev: &'a LogEvent, group: &str) -> Option<&'a str> {
    match group {
        "level" => Some(ev.level.as_str()),
        other => ev.fields.get(other).map(|s| s.as_str()),
    }
}

pub fn apply(filter: &Filter, events: &[LogEvent], source: &FileSource) -> Vec<u32> {
    // `events` is reverse-chronological (events[0] is newest). The UI wants
    // the newest at the BOTTOM of the list, so we emit indices oldest-first:
    // visible[0] = oldest, visible[last] = newest.
    if filter.is_empty() {
        return (0..events.len() as u32).rev().collect();
    }
    events
        .iter()
        .enumerate()
        .rev()
        .filter(|(_, e)| filter.matches(e, source))
        .map(|(i, _)| i as u32)
        .collect()
}

fn contains_ci(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    let h = haystack.as_bytes();
    let n = needle.as_bytes();
    if n.len() > h.len() {
        return false;
    }
    // ASCII case-insensitive substring search without per-line allocation.
    if haystack.is_ascii() && needle.is_ascii() {
        let last = h.len() - n.len();
        'outer: for i in 0..=last {
            for j in 0..n.len() {
                if !h[i + j].eq_ignore_ascii_case(&n[j]) {
                    continue 'outer;
                }
            }
            return true;
        }
        false
    } else {
        haystack.to_lowercase().contains(&needle.to_lowercase())
    }
}
