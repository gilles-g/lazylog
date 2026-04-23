use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct MainLayout {
    pub facets: Rect,
    pub content: Rect,
    pub status: Rect,
}

pub fn main_layout(area: Rect) -> MainLayout {
    let vchunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);
    let hchunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(10)])
        .split(vchunks[0]);
    MainLayout {
        facets: hchunks[0],
        content: hchunks[1],
        status: vchunks[1],
    }
}

pub fn split_content_with_detail(area: Rect, show_detail: bool) -> (Rect, Option<Rect>) {
    if !show_detail {
        return (area, None);
    }
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
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
