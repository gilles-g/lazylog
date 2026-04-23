use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;

use crate::log::event::LogEvent;
use crate::ui::style::theme;

pub struct EventsPanel;

impl EventsPanel {
    pub fn render(
        frame: &mut Frame<'_>,
        area: Rect,
        events: &[LogEvent],
        visible: &[u32],
        cursor: usize,
        focused: bool,
    ) {
        let border = theme::border(focused);
        let title = format!(" Events ({}) ", visible.len());
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border)
            .title(title);

        if visible.is_empty() {
            frame.render_widget(
                ratatui::widgets::Paragraph::new("No matching events. Press r to reset filters.")
                    .block(block),
                area,
            );
            return;
        }

        let inner = block.inner(area);
        let capacity = inner.height.max(1) as usize;
        let max_start = visible.len().saturating_sub(capacity);
        let start = cursor.saturating_sub(capacity / 2).min(max_start);
        let end = (start + capacity).min(visible.len());

        let items: Vec<ListItem> = visible[start..end]
            .iter()
            .map(|&idx| {
                let ev = &events[idx as usize];
                let ts = ev
                    .timestamp
                    .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "─ no ts ─".to_string());
                let lvl = ev.level.as_str();
                let source = ev.source.as_deref().unwrap_or("-");
                let msg = truncate(&ev.message, inner.width as usize);
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{ts} "), Style::default().fg(Color::DarkGray)),
                    Span::styled(format!("{lvl:<8} "), theme::level_style(ev.level)),
                    Span::styled(format!("{source:<12} "), Style::default().fg(Color::Blue)),
                    Span::raw(msg),
                ]))
            })
            .collect();

        let mut state = ListState::default();
        state.select(Some(cursor.saturating_sub(start)));
        let list = List::new(items)
            .block(block)
            .highlight_style(theme::selected_row())
            .highlight_symbol("▶ ");
        frame.render_stateful_widget(list, area, &mut state);
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}
