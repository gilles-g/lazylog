use ratatui::style::{Color, Modifier, Style};

use crate::log::event::Level;

pub const BG: Color = Color::Reset;
pub const FG: Color = Color::White;
pub const BORDER: Color = Color::DarkGray;
pub const BORDER_ACTIVE: Color = Color::Cyan;
pub const ACCENT: Color = Color::Cyan;
pub const DIM: Color = Color::DarkGray;
pub const HIGHLIGHT_BG: Color = Color::Rgb(40, 60, 80);

pub fn level_style(level: Level) -> Style {
    let base = Style::default();
    match level {
        Level::Debug => base.fg(Color::DarkGray),
        Level::Info => base.fg(Color::Cyan),
        Level::Notice => base.fg(Color::Blue),
        Level::Warning => base.fg(Color::Yellow),
        Level::Error => base.fg(Color::Red),
        Level::Critical | Level::Alert | Level::Emergency => {
            base.fg(Color::Red).add_modifier(Modifier::BOLD)
        }
        Level::Unknown => base.fg(Color::Gray),
    }
}

pub fn selected_row() -> Style {
    Style::default()
        .bg(HIGHLIGHT_BG)
        .add_modifier(Modifier::BOLD)
}

pub fn border(active: bool) -> Style {
    if active {
        Style::default().fg(BORDER_ACTIVE)
    } else {
        Style::default().fg(BORDER)
    }
}

pub fn dim() -> Style {
    Style::default().fg(DIM)
}

pub fn accent() -> Style {
    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
}
