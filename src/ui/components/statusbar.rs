use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::log::format::LogFormat;

pub struct StatusLine<'a> {
    pub format: LogFormat,
    pub total: usize,
    pub visible: usize,
    pub path: &'a str,
    pub loading: bool,
    pub cancelled: bool,
    pub toast: Option<(&'a str, bool)>,
}

pub fn render(frame: &mut Frame<'_>, area: Rect, s: StatusLine<'_>) {
    let mut spans = vec![
        Span::styled(
            " lazylog ",
            Style::default().bg(Color::Cyan).fg(Color::Black),
        ),
        Span::raw(" "),
        Span::styled(s.format.label(), Style::default().fg(Color::Yellow)),
        Span::raw("  "),
        Span::styled(
            format!("{}/{}", s.visible, s.total),
            Style::default().fg(Color::White),
        ),
        Span::raw(" events  "),
        Span::styled(s.path, Style::default().fg(Color::DarkGray)),
    ];
    if s.loading {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            "loading… (Esc to stop)",
            Style::default().fg(Color::LightYellow),
        ));
    } else if s.cancelled {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            "partial (stopped)",
            Style::default().fg(Color::LightRed),
        ));
    }
    if let Some((msg, is_err)) = s.toast {
        spans.push(Span::raw("  "));
        let color = if is_err { Color::LightRed } else { Color::LightGreen };
        spans.push(Span::styled(msg.to_string(), Style::default().fg(color)));
    }
    spans.push(Span::raw("   "));
    spans.push(Span::styled("?", Style::default().fg(Color::Cyan)));
    spans.push(Span::styled(
        " help  ",
        Style::default().fg(Color::DarkGray),
    ));
    spans.push(Span::styled("q", Style::default().fg(Color::Cyan)));
    spans.push(Span::styled(" quit", Style::default().fg(Color::DarkGray)));
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}
