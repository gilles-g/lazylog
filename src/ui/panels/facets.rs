use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;

use crate::query::facets::FacetIndex;
use crate::query::filter::Filter;
use crate::ui::style::theme;

pub struct FacetsPanel;

/// Flat index: (group_idx, value_idx). We flatten for keyboard navigation.
#[derive(Debug, Clone, Copy, Default)]
pub struct FacetCursor {
    pub flat: usize,
}

impl FacetCursor {
    pub fn locate(&self, index: &FacetIndex) -> Option<(usize, usize)> {
        let mut cursor = 0;
        for (g, group) in index.groups.iter().enumerate() {
            for v in 0..group.values.len() {
                if cursor == self.flat {
                    return Some((g, v));
                }
                cursor += 1;
            }
        }
        None
    }

    pub fn total(index: &FacetIndex) -> usize {
        index.groups.iter().map(|g| g.values.len()).sum()
    }
}

impl FacetsPanel {
    pub fn render(
        frame: &mut Frame<'_>,
        area: Rect,
        facets: &FacetIndex,
        filter: &Filter,
        cursor: FacetCursor,
        focused: bool,
    ) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme::border(focused))
            .title(" Facets ");

        let mut items: Vec<ListItem> = Vec::new();
        for group in &facets.groups {
            items.push(ListItem::new(Line::from(Span::styled(
                format!(" {} ", group.label),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ))));
            let selected_set = filter.selections.get(&group.key);
            for (value, count) in &group.values {
                let is_selected = selected_set.map(|s| s.contains(value)).unwrap_or(false);
                let marker = if is_selected { "■" } else { "□" };
                let style = if is_selected {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default().fg(Color::White)
                };
                items.push(ListItem::new(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(marker, style),
                    Span::raw(" "),
                    Span::styled(truncate(value, 16), style),
                    Span::raw(" "),
                    Span::styled(format!("({count})"), Style::default().fg(Color::DarkGray)),
                ])));
            }
            if group.distinct > group.values.len() {
                let extra = group.distinct - group.values.len();
                items.push(ListItem::new(Line::from(vec![
                    Span::raw("   "),
                    Span::styled(
                        format!("… +{extra} more (use / to narrow)"),
                        Style::default().fg(Color::DarkGray),
                    ),
                ])));
            }
        }

        // Compute selected flat index → list item index (accounting for group header rows).
        let selected_row = cursor.locate(facets).map(|(g_idx, v_idx)| {
            let mut row = 0_usize;
            for (gi, group) in facets.groups.iter().enumerate() {
                row += 1; // header
                if gi == g_idx {
                    row += v_idx;
                    break;
                }
                row += group.values.len();
                if group.distinct > group.values.len() {
                    row += 1; // "+N more" line
                }
            }
            row
        });

        // Manual viewport slicing: centering the cursor ourselves avoids
        // ratatui's auto-scroll glueing the selection to the bottom edge when
        // navigating upward with a freshly-built ListState.
        let inner = block.inner(area);
        let capacity = inner.height.max(1) as usize;
        let total = items.len();
        let max_start = total.saturating_sub(capacity);
        let start = selected_row
            .map(|r| r.saturating_sub(capacity / 2).min(max_start))
            .unwrap_or(0);
        let end = (start + capacity).min(total);
        let window: Vec<ListItem> = items.into_iter().skip(start).take(end - start).collect();

        let mut list_state = ListState::default();
        if let Some(row) = selected_row {
            list_state.select(Some(row.saturating_sub(start)));
        }
        let list = List::new(window)
            .block(block)
            .highlight_style(theme::selected_row())
            .highlight_symbol("▶ ");
        frame.render_stateful_widget(list, area, &mut list_state);
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
