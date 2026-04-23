use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

#[derive(Debug, Clone, Default)]
pub struct InputBox {
    pub title: String,
    pub buffer: String,
    pub active: bool,
}

impl InputBox {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            buffer: String::new(),
            active: false,
        }
    }

    pub fn push(&mut self, c: char) {
        self.buffer.push(c);
    }

    pub fn backspace(&mut self) {
        self.buffer.pop();
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn render(&self, frame: &mut Frame<'_>, area: Rect) {
        let border = if self.active {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border)
            .title(self.title.as_str());
        frame.render_widget(Paragraph::new(self.buffer.as_str()).block(block), area);
    }
}
