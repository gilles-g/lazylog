use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::log::event::LogEvent;
use crate::log::source::FileSource;
use crate::ui::style::theme;

pub struct DetailPanel;

impl DetailPanel {
    pub fn render(
        frame: &mut Frame<'_>,
        area: Rect,
        event: Option<&LogEvent>,
        source: Option<&FileSource>,
    ) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme::border(false))
            .title(" Detail ");
        let ev = match event {
            Some(e) => e,
            None => {
                frame.render_widget(Paragraph::new("").block(block), area);
                return;
            }
        };

        let mut lines: Vec<Line> = Vec::new();
        lines.push(Line::from(vec![
            Span::styled("level   ", Style::default().fg(Color::DarkGray)),
            Span::styled(ev.level.as_str().to_string(), theme::level_style(ev.level)),
        ]));
        if let Some(ts) = ev.timestamp {
            lines.push(Line::from(vec![
                Span::styled("time    ", Style::default().fg(Color::DarkGray)),
                Span::raw(ts.format("%Y-%m-%d %H:%M:%S %:z").to_string()),
            ]));
        }
        if let Some(src) = &ev.source {
            lines.push(Line::from(vec![
                Span::styled("source  ", Style::default().fg(Color::DarkGray)),
                Span::styled(src.clone(), Style::default().fg(Color::Blue)),
            ]));
        }
        lines.push(Line::from(vec![
            Span::styled("line    ", Style::default().fg(Color::DarkGray)),
            Span::raw(ev.line_no.to_string()),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "message",
            Style::default().fg(Color::Cyan),
        )));
        lines.push(Line::from(ev.message.clone()));

        if !ev.fields.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "fields",
                Style::default().fg(Color::Cyan),
            )));
            for (k, v) in &ev.fields {
                lines.push(Line::from(vec![
                    Span::styled(format!("  {k:<12}"), Style::default().fg(Color::DarkGray)),
                    Span::raw(pretty(v)),
                ]));
            }
        }

        if let Some(src) = source {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "raw",
                Style::default().fg(Color::Cyan),
            )));
            lines.push(Line::from(Span::styled(
                src.slice(ev.offset, ev.len).to_string(),
                Style::default().fg(Color::DarkGray),
            )));
        }

        frame.render_widget(
            Paragraph::new(lines)
                .wrap(Wrap { trim: false })
                .block(block),
            area,
        );
    }
}

fn pretty(v: &str) -> String {
    let t = v.trim();
    if (t.starts_with('{') && t.ends_with('}')) || (t.starts_with('[') && t.ends_with(']')) {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(t) {
            if let Ok(pp) = serde_json::to_string_pretty(&parsed) {
                return pp;
            }
        }
    }
    v.to_string()
}
