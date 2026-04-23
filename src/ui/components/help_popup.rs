use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::ui::keys::HELP;
use crate::ui::layout::centered;

pub fn render(frame: &mut Frame<'_>, area: Rect) {
    let w = 60_u16.min(area.width.saturating_sub(4));
    let h = (HELP.len() as u16 + 4).min(area.height.saturating_sub(4));
    let rect = centered(area, w, h);
    frame.render_widget(Clear, rect);

    let lines: Vec<Line> = HELP
        .iter()
        .map(|b| {
            Line::from(vec![
                Span::styled(
                    format!("{:<16}", b.keys),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(b.desc),
            ])
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Help — press ? or Esc to close ");
    frame.render_widget(Paragraph::new(lines).block(block), rect);
}
