use chrono::{DateTime, Duration, FixedOffset, NaiveDate, NaiveDateTime, Utc};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::ui::layout::centered;

pub type DateBound = Option<DateTime<FixedOffset>>;
pub type DateRange = (DateBound, DateBound);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    All,
    Last15m,
    Last1h,
    Last24h,
    Last7d,
    Custom,
}

impl Preset {
    pub fn label(&self) -> &'static str {
        match self {
            Preset::All => "All time",
            Preset::Last15m => "Last 15 minutes",
            Preset::Last1h => "Last 1 hour",
            Preset::Last24h => "Last 24 hours",
            Preset::Last7d => "Last 7 days",
            Preset::Custom => "Custom range…",
        }
    }

    pub fn range(&self, reference: DateTime<FixedOffset>) -> DateRange {
        match self {
            Preset::All => (None, None),
            Preset::Last15m => (Some(reference - Duration::minutes(15)), None),
            Preset::Last1h => (Some(reference - Duration::hours(1)), None),
            Preset::Last24h => (Some(reference - Duration::hours(24)), None),
            Preset::Last7d => (Some(reference - Duration::days(7)), None),
            Preset::Custom => (None, None),
        }
    }

    pub fn all() -> [Preset; 6] {
        [
            Preset::All,
            Preset::Last15m,
            Preset::Last1h,
            Preset::Last24h,
            Preset::Last7d,
            Preset::Custom,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Presets,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomField {
    From,
    To,
}

pub struct DateRangeModal {
    pub cursor: usize,
    pub mode: Mode,
    pub field: CustomField,
    pub from_input: String,
    pub to_input: String,
    pub error: Option<String>,
}

impl Default for DateRangeModal {
    fn default() -> Self {
        Self {
            cursor: 0,
            mode: Mode::Presets,
            field: CustomField::From,
            from_input: String::new(),
            to_input: String::new(),
            error: None,
        }
    }
}

impl DateRangeModal {
    pub fn up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn down(&mut self) {
        if self.cursor + 1 < Preset::all().len() {
            self.cursor += 1;
        }
    }

    pub fn selected(&self) -> Preset {
        Preset::all()[self.cursor]
    }

    pub fn reference_now() -> DateTime<FixedOffset> {
        Utc::now().with_timezone(&FixedOffset::east_opt(0).unwrap())
    }

    pub fn enter_custom(&mut self) {
        self.mode = Mode::Custom;
        self.field = CustomField::From;
        self.error = None;
    }

    pub fn exit_custom(&mut self) {
        self.mode = Mode::Presets;
        self.error = None;
    }

    pub fn toggle_field(&mut self) {
        self.field = match self.field {
            CustomField::From => CustomField::To,
            CustomField::To => CustomField::From,
        };
    }

    fn active_buffer_mut(&mut self) -> &mut String {
        match self.field {
            CustomField::From => &mut self.from_input,
            CustomField::To => &mut self.to_input,
        }
    }

    pub fn push(&mut self, c: char) {
        self.active_buffer_mut().push(c);
        self.error = None;
    }

    pub fn backspace(&mut self) {
        self.active_buffer_mut().pop();
        self.error = None;
    }

    pub fn parse_custom(&mut self) -> Option<DateRange> {
        let from = match parse_bound(&self.from_input) {
            Ok(v) => v,
            Err(e) => {
                self.error = Some(format!("from: {e}"));
                return None;
            }
        };
        let to = match parse_bound(&self.to_input) {
            Ok(v) => v,
            Err(e) => {
                self.error = Some(format!("to: {e}"));
                return None;
            }
        };
        if let (Some(f), Some(t)) = (from, to) {
            if f > t {
                self.error = Some("'from' must be earlier than 'to'".into());
                return None;
            }
        }
        self.error = None;
        Some((from, to))
    }

    pub fn render(&self, frame: &mut Frame<'_>, area: Rect) {
        match self.mode {
            Mode::Presets => self.render_presets(frame, area),
            Mode::Custom => self.render_custom(frame, area),
        }
    }

    fn render_presets(&self, frame: &mut Frame<'_>, area: Rect) {
        let w = 40_u16.min(area.width.saturating_sub(4));
        let h = (Preset::all().len() as u16 + 4).min(area.height.saturating_sub(4));
        let rect = centered(area, w, h);
        frame.render_widget(Clear, rect);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Date range — Enter to apply, Esc to cancel ");

        let items: Vec<ListItem> = Preset::all()
            .iter()
            .map(|p| {
                ListItem::new(Line::from(Span::styled(
                    format!("  {}", p.label()),
                    Style::default().fg(Color::White),
                )))
            })
            .collect();

        let mut state = ListState::default();
        state.select(Some(self.cursor));
        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .bg(Color::Cyan)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");
        frame.render_stateful_widget(list, rect, &mut state);
    }

    fn render_custom(&self, frame: &mut Frame<'_>, area: Rect) {
        let w = 64_u16.min(area.width.saturating_sub(4));
        let h = 11_u16.min(area.height.saturating_sub(4));
        let rect = centered(area, w, h);
        frame.render_widget(Clear, rect);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Custom range — Tab to switch, Enter to apply, Esc to go back ");
        let inner = block.inner(rect);
        frame.render_widget(block, rect);

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Min(1),
            ])
            .split(inner);

        let from_active = matches!(self.field, CustomField::From);
        let to_active = matches!(self.field, CustomField::To);
        frame.render_widget(field_widget("From", &self.from_input, from_active), rows[0]);
        frame.render_widget(field_widget("To", &self.to_input, to_active), rows[1]);

        let hint =
            Paragraph::new("Empty = no bound · YYYY-MM-DD · YYYY-MM-DD HH:MM[:SS] · RFC3339")
                .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(hint, rows[2]);

        if let Some(err) = &self.error {
            let e = Paragraph::new(err.as_str()).style(Style::default().fg(Color::Red));
            frame.render_widget(e, rows[3]);
        }
    }
}

fn field_widget<'a>(title: &'a str, value: &'a str, active: bool) -> Paragraph<'a> {
    let border = if active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border)
        .title(format!(" {title} "));
    let text = if active {
        format!("{value}_")
    } else {
        value.to_string()
    };
    Paragraph::new(text).block(block)
}

pub fn parse_bound(input: &str) -> Result<DateBound, String> {
    let s = input.trim();
    if s.is_empty() {
        return Ok(None);
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(Some(dt));
    }
    let tz0 = FixedOffset::east_opt(0).unwrap();
    if let Ok(n) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Ok(Some(DateTime::from_naive_utc_and_offset(n, tz0)));
    }
    if let Ok(n) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M") {
        return Ok(Some(DateTime::from_naive_utc_and_offset(n, tz0)));
    }
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let n = d.and_hms_opt(0, 0, 0).unwrap();
        return Ok(Some(DateTime::from_naive_utc_and_offset(n, tz0)));
    }
    Err("expected YYYY-MM-DD, YYYY-MM-DD HH:MM[:SS] or RFC3339".into())
}
