use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;

use crate::log::scanner::CandidateLog;
use crate::ui::style::theme;

pub struct PickerState {
    pub candidates: Vec<CandidateLog>,
    pub cursor: usize,
}

pub enum PickerOutcome {
    Pending,
    Selected(PathBuf),
    Cancelled,
}

impl PickerState {
    pub fn new(candidates: Vec<CandidateLog>) -> Self {
        Self {
            candidates,
            cursor: 0,
        }
    }

    pub fn handle_key(&mut self, ev: KeyEvent) -> PickerOutcome {
        match ev.code {
            KeyCode::Esc | KeyCode::Char('q') => PickerOutcome::Cancelled,
            KeyCode::Down | KeyCode::Char('j') => {
                if self.cursor + 1 < self.candidates.len() {
                    self.cursor += 1;
                }
                PickerOutcome::Pending
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
                PickerOutcome::Pending
            }
            KeyCode::Enter => self
                .candidates
                .get(self.cursor)
                .map(|c| PickerOutcome::Selected(c.path.clone()))
                .unwrap_or(PickerOutcome::Pending),
            _ => PickerOutcome::Pending,
        }
    }

    pub fn render(&self, frame: &mut Frame<'_>, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Choose a log file — Enter to open, q to quit ");

        if self.candidates.is_empty() {
            frame.render_widget(
                ratatui::widgets::Paragraph::new(
                    "No log files detected in var/log, logs/, or /var/log.\n\nPass a path as argument: lazylog /path/to/file.log",
                )
                .block(block),
                area,
            );
            return;
        }

        let items: Vec<ListItem> = self
            .candidates
            .iter()
            .map(|c| {
                let size = humansize(c.size);
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{:>10}  ", size),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        c.path.display().to_string(),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::empty()),
                    ),
                ]))
            })
            .collect();

        let mut state = ListState::default();
        state.select(Some(self.cursor));
        let list = List::new(items)
            .block(block)
            .highlight_style(theme::selected_row())
            .highlight_symbol("▶ ");
        frame.render_stateful_widget(list, area, &mut state);
    }
}

fn humansize(n: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut f = n as f64;
    let mut u = 0;
    while f >= 1024.0 && u < UNITS.len() - 1 {
        f /= 1024.0;
        u += 1;
    }
    if u == 0 {
        format!("{n} {}", UNITS[0])
    } else {
        format!("{f:.1} {}", UNITS[u])
    }
}
