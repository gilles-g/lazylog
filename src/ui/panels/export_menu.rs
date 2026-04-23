use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState};
use ratatui::Frame;

use crate::ui::layout::centered;
use crate::ui::style::theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportChoice {
    FacetGroup,
    FilteredLog,
}

impl ExportChoice {
    const ALL: [ExportChoice; 2] = [ExportChoice::FacetGroup, ExportChoice::FilteredLog];

    fn label(self) -> &'static str {
        match self {
            ExportChoice::FacetGroup => "Focused facet group (value\\tcount)",
            ExportChoice::FilteredLog => "Filtered log lines (raw)",
        }
    }
}

#[derive(Default)]
pub struct ExportMenu {
    pub cursor: usize,
}

impl ExportMenu {
    pub fn up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }
    pub fn down(&mut self) {
        if self.cursor + 1 < ExportChoice::ALL.len() {
            self.cursor += 1;
        }
    }
    pub fn selected(&self) -> ExportChoice {
        ExportChoice::ALL[self.cursor.min(ExportChoice::ALL.len() - 1)]
    }

    pub fn render(&self, frame: &mut Frame<'_>, area: Rect) {
        let rect = centered(area, 60, 7);
        frame.render_widget(Clear, rect);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Export — Enter to confirm, Esc to cancel ");

        let items: Vec<ListItem> = ExportChoice::ALL
            .iter()
            .map(|c| {
                ListItem::new(Line::from(vec![Span::styled(
                    format!("  {}", c.label()),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::empty()),
                )]))
            })
            .collect();

        let mut state = ListState::default();
        state.select(Some(self.cursor));
        let list = List::new(items)
            .block(block)
            .highlight_style(theme::selected_row())
            .highlight_symbol("▶ ");
        frame.render_stateful_widget(list, rect, &mut state);
    }
}
