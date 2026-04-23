use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub const DEFAULT_FACETS_RATIO: f64 = 0.22;
pub const DEFAULT_EVENTS_RATIO: f64 = 0.60;
const MIN_RATIO: f64 = 0.10;
const MAX_RATIO: f64 = 0.80;
const RATIO_STEP: f64 = 0.02;

pub struct MainLayout {
    pub facets: Rect,
    pub content: Rect,
    pub status: Rect,
}

pub fn main_layout(area: Rect, facets_ratio: f64) -> MainLayout {
    let vchunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);
    let mut facets_w = (vchunks[0].width as f64 * facets_ratio) as u16;
    if facets_w < 12 {
        facets_w = 12.min(vchunks[0].width);
    }
    let hchunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(facets_w), Constraint::Min(10)])
        .split(vchunks[0]);
    MainLayout {
        facets: hchunks[0],
        content: hchunks[1],
        status: vchunks[1],
    }
}

pub fn split_content_with_detail(
    area: Rect,
    show_detail: bool,
    events_ratio: f64,
) -> (Rect, Option<Rect>) {
    if !show_detail {
        return (area, None);
    }
    let pct = (events_ratio * 100.0).round().clamp(10.0, 90.0) as u16;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(pct),
            Constraint::Percentage(100 - pct),
        ])
        .split(area);
    (chunks[0], Some(chunks[1]))
}

pub fn centered(area: Rect, w: u16, h: u16) -> Rect {
    let w = w.min(area.width);
    let h = h.min(area.height);
    let x = area.x + (area.width - w) / 2;
    let y = area.y + (area.height - h) / 2;
    Rect::new(x, y, w, h)
}

/// Clamps and adjusts a ratio by the standard step.
pub fn adjust_ratio(current: f64, increase: bool) -> f64 {
    let new = if increase {
        current + RATIO_STEP
    } else {
        current - RATIO_STEP
    };
    new.clamp(MIN_RATIO, MAX_RATIO)
}
