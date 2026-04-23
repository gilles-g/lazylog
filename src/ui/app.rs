use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::backend::Backend;
use ratatui::Terminal;

use crate::log::event::LogEvent;
use crate::log::format::LogFormat;
use crate::log::geoip::GeoDb;
use crate::log::loader::{self, DateFilter, LoadHandle, LoadMsg};
use crate::log::source::FileSource;
use crate::query::export;
use crate::query::facets::{self, FacetIndex};
use crate::query::filter::{self, Filter};
use crate::ui::components::{help_popup, input_box::InputBox, statusbar};
use crate::ui::keys::{Action, KeyMap};
use crate::ui::layout::{main_layout, split_content_with_detail};
use crate::ui::panels::daterange::{DateRangeModal, Mode as DateRangeMode, Preset};
use crate::ui::panels::detail::DetailPanel;
use crate::ui::panels::events::EventsPanel;
use crate::ui::panels::export_menu::{ExportChoice, ExportMenu};
use crate::ui::panels::facets::{FacetCursor, FacetsPanel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Facets,
    Events,
}

pub struct App {
    path: PathBuf,
    format: LogFormat,
    source: Option<Arc<FileSource>>,
    events: Vec<LogEvent>,
    visible: Vec<u32>,
    facets: FacetIndex,
    filter: Filter,

    focus: Focus,
    event_cursor: usize,
    facet_cursor: FacetCursor,
    show_help: bool,
    show_daterange: bool,
    date_modal: DateRangeModal,
    show_export: bool,
    export_menu: ExportMenu,

    search: InputBox,

    load: Option<LoadHandle>,
    loading: bool,
    cancelled: bool,
    last_recompute: Instant,
    pending_dirty: bool,

    /// When true, the cursor snaps to the most recent event (last row of the
    /// visible list, since newest is displayed at the bottom).
    auto_tail: bool,

    toast: Option<(String, bool, Instant)>,

    quit: bool,
}

const RECOMPUTE_DEBOUNCE: Duration = Duration::from_millis(400);
const TOAST_TTL: Duration = Duration::from_secs(4);

impl App {
    pub fn new(
        path: PathBuf,
        format: LogFormat,
        initial_filter: Filter,
        geo: Option<Arc<GeoDb>>,
    ) -> Self {
        let date_filter = DateFilter {
            from: initial_filter.from,
            to: initial_filter.to,
        };
        let load = loader::load(&path, format, date_filter, geo);
        Self {
            path,
            format,
            source: None,
            events: Vec::new(),
            visible: Vec::new(),
            facets: FacetIndex::default(),
            filter: initial_filter,
            focus: Focus::Events,
            event_cursor: 0,
            facet_cursor: FacetCursor::default(),
            show_help: false,
            show_daterange: false,
            date_modal: DateRangeModal::default(),
            show_export: false,
            export_menu: ExportMenu::default(),
            search: InputBox::new(" search (Enter to apply, Esc to close) "),
            load: Some(load),
            loading: true,
            cancelled: false,
            last_recompute: Instant::now(),
            pending_dirty: false,
            auto_tail: true,
            toast: None,
            quit: false,
        }
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        while !self.quit {
            terminal.draw(|frame| self.draw(frame))?;
            self.drain_loader();
            self.maybe_recompute();
            if event::poll(Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(key) if key.kind != KeyEventKind::Release => {
                        self.handle_key(key);
                    }
                    Event::Resize(_, _) => {}
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn drain_loader(&mut self) {
        let Some(load) = self.load.as_ref() else {
            return;
        };
        let mut dirty = false;
        let mut done = false;
        loop {
            match load.rx.try_recv() {
                Ok(LoadMsg::Source(src)) => {
                    self.source = Some(src);
                }
                Ok(LoadMsg::Chunk(mut chunk)) => {
                    self.events.append(&mut chunk);
                    dirty = true;
                }
                Ok(LoadMsg::Done { cancelled, .. }) => {
                    self.loading = false;
                    self.cancelled = cancelled;
                    done = true;
                    dirty = true;
                    break;
                }
                Ok(LoadMsg::Error(e)) => {
                    log::error!("loader error: {e}");
                    self.loading = false;
                    done = true;
                    break;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.loading = false;
                    done = true;
                    break;
                }
            }
        }
        if done {
            self.load = None;
        }
        if dirty {
            self.pending_dirty = true;
        }
    }

    fn maybe_recompute(&mut self) {
        if !self.pending_dirty {
            return;
        }
        // During loading we debounce. When finished, recompute immediately so
        // the final state is accurate.
        if self.loading && self.last_recompute.elapsed() < RECOMPUTE_DEBOUNCE {
            return;
        }
        self.recompute();
        self.pending_dirty = false;
        self.last_recompute = Instant::now();
    }

    fn recompute(&mut self) {
        let Some(source) = self.source.as_ref() else {
            return;
        };
        self.visible = filter::apply(&self.filter, &self.events, source);
        self.facets = facets::compute(self.format, &self.events, &self.visible);
        if self.auto_tail && !self.visible.is_empty() {
            self.event_cursor = self.visible.len() - 1;
        } else if self.event_cursor >= self.visible.len() {
            self.event_cursor = self.visible.len().saturating_sub(1);
        }
        let total = FacetCursor::total(&self.facets);
        if self.facet_cursor.flat >= total {
            self.facet_cursor.flat = total.saturating_sub(1);
        }
    }

    fn force_recompute(&mut self) {
        self.pending_dirty = false;
        self.recompute();
        self.last_recompute = Instant::now();
    }

    fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        if self.search.active {
            match key.code {
                KeyCode::Esc => {
                    self.search.active = false;
                    self.search.clear();
                    self.filter.text.clear();
                    self.force_recompute();
                }
                KeyCode::Enter => {
                    self.filter.text = self.search.buffer.clone();
                    self.search.active = false;
                    self.force_recompute();
                }
                KeyCode::Backspace => {
                    self.search.backspace();
                }
                KeyCode::Char(c) => {
                    self.search.push(c);
                }
                _ => {}
            }
            return;
        }
        if self.show_export {
            match key.code {
                KeyCode::Esc => self.show_export = false,
                KeyCode::Up | KeyCode::Char('k') => self.export_menu.up(),
                KeyCode::Down | KeyCode::Char('j') => self.export_menu.down(),
                KeyCode::Enter => {
                    let choice = self.export_menu.selected();
                    self.show_export = false;
                    self.run_export(choice);
                }
                _ => {}
            }
            return;
        }
        if self.show_daterange {
            match self.date_modal.mode {
                DateRangeMode::Presets => match key.code {
                    KeyCode::Esc => self.show_daterange = false,
                    KeyCode::Up | KeyCode::Char('k') => self.date_modal.up(),
                    KeyCode::Down | KeyCode::Char('j') => self.date_modal.down(),
                    KeyCode::Enter => {
                        if self.date_modal.selected() == Preset::Custom {
                            self.date_modal.enter_custom();
                        } else {
                            let (from, to) = self
                                .date_modal
                                .selected()
                                .range(DateRangeModal::reference_now());
                            self.filter.from = from;
                            self.filter.to = to;
                            self.show_daterange = false;
                            self.force_recompute();
                        }
                    }
                    _ => {}
                },
                DateRangeMode::Custom => match key.code {
                    KeyCode::Esc => self.date_modal.exit_custom(),
                    KeyCode::Tab | KeyCode::BackTab | KeyCode::Up | KeyCode::Down => {
                        self.date_modal.toggle_field();
                    }
                    KeyCode::Enter => {
                        if let Some((from, to)) = self.date_modal.parse_custom() {
                            self.filter.from = from;
                            self.filter.to = to;
                            self.show_daterange = false;
                            self.force_recompute();
                        }
                    }
                    KeyCode::Backspace => self.date_modal.backspace(),
                    KeyCode::Char(c) => self.date_modal.push(c),
                    _ => {}
                },
            }
            return;
        }

        let action = match KeyMap::map(key) {
            Some(a) => a,
            None => return,
        };
        // Esc while loading → cancel the background parse (user has seen enough).
        if self.loading && matches!(action, Action::CloseOverlay) {
            if let Some(load) = self.load.as_ref() {
                load.cancel();
            }
            return;
        }
        match action {
            Action::Quit => self.quit = true,
            Action::Help => self.show_help = !self.show_help,
            Action::FocusFacets => self.focus = Focus::Facets,
            Action::FocusEvents => self.focus = Focus::Events,
            Action::OpenSearch => {
                self.search.active = true;
                self.search.buffer = self.filter.text.clone();
            }
            Action::OpenDateRange => {
                self.show_daterange = true;
            }
            Action::OpenDetail => {}
            Action::CloseOverlay => {
                self.show_help = false;
                self.show_daterange = false;
                self.show_export = false;
            }
            Action::ResetFilters => {
                self.filter.reset();
                self.search.clear();
                self.auto_tail = true;
                self.force_recompute();
            }
            Action::Up => {
                self.disable_tail_on_events();
                self.move_cursor(-1);
            }
            Action::Down => {
                self.disable_tail_on_events();
                self.move_cursor(1);
            }
            Action::PageUp => {
                self.disable_tail_on_events();
                self.move_cursor(-10);
            }
            Action::PageDown => {
                self.disable_tail_on_events();
                self.move_cursor(10);
            }
            Action::Top => {
                // g: oldest (top of list)
                self.disable_tail_on_events();
                self.jump(0);
            }
            Action::Bottom => {
                // G: newest (bottom of list) — re-arms auto-tail on Events.
                if matches!(self.focus, Focus::Events) {
                    self.auto_tail = true;
                }
                self.jump(isize::MAX);
            }
            Action::ToggleSelection => self.toggle_facet(),
            Action::ExportFacet => {
                self.export_menu = ExportMenu::default();
                self.show_export = true;
            }
        }
    }

    fn run_export(&mut self, choice: ExportChoice) {
        let result = match choice {
            ExportChoice::FacetGroup => {
                let Some(group_key) = self.focused_group_key() else {
                    self.set_toast(
                        "export: move cursor onto a facet value (press f, then j/k)".into(),
                        true,
                    );
                    return;
                };
                export::export_group(&self.path, &group_key, &self.events, &self.visible)
            }
            ExportChoice::FilteredLog => {
                let Some(source) = self.source.as_deref() else {
                    self.set_toast("export: log source not ready yet".into(), true);
                    return;
                };
                if self.visible.is_empty() {
                    self.set_toast("export: nothing visible to export".into(), true);
                    return;
                }
                export::export_filtered(&self.path, &self.events, &self.visible, source)
            }
        };
        match result {
            Ok(path) => {
                let absolute = std::fs::canonicalize(&path).unwrap_or(path);
                self.set_toast(format!("exported → {}", absolute.display()), false);
            }
            Err(e) => self.set_toast(format!("export failed: {e}"), true),
        }
    }

    fn focused_group_key(&self) -> Option<String> {
        let (g, _) = self.facet_cursor.locate(&self.facets)?;
        Some(self.facets.groups[g].key.clone())
    }

    fn set_toast(&mut self, msg: String, is_err: bool) {
        self.toast = Some((msg, is_err, Instant::now()));
    }

    fn disable_tail_on_events(&mut self) {
        if matches!(self.focus, Focus::Events) {
            self.auto_tail = false;
        }
    }

    fn move_cursor(&mut self, delta: isize) {
        match self.focus {
            Focus::Events => {
                let len = self.visible.len();
                if len == 0 {
                    return;
                }
                let new = (self.event_cursor as isize + delta).clamp(0, (len - 1) as isize);
                self.event_cursor = new as usize;
            }
            Focus::Facets => {
                let total = FacetCursor::total(&self.facets);
                if total == 0 {
                    return;
                }
                let new = (self.facet_cursor.flat as isize + delta).clamp(0, (total - 1) as isize);
                self.facet_cursor.flat = new as usize;
            }
        }
    }

    fn jump(&mut self, target: isize) {
        match self.focus {
            Focus::Events => {
                let len = self.visible.len();
                if len == 0 {
                    return;
                }
                self.event_cursor = target.clamp(0, (len - 1) as isize) as usize;
            }
            Focus::Facets => {
                let total = FacetCursor::total(&self.facets);
                if total == 0 {
                    return;
                }
                self.facet_cursor.flat = target.clamp(0, (total - 1) as isize) as usize;
            }
        }
    }

    fn toggle_facet(&mut self) {
        let Some((g, v)) = self.facet_cursor.locate(&self.facets) else {
            return;
        };
        let group = &self.facets.groups[g];
        let value = group.values[v].0.clone();
        let key = group.key.clone();
        self.filter.toggle(&key, &value);
        self.force_recompute();
    }

    fn draw(&mut self, frame: &mut ratatui::Frame<'_>) {
        let area = frame.area();
        let layout = main_layout(area);

        FacetsPanel::render(
            frame,
            layout.facets,
            &self.facets,
            &self.filter,
            self.facet_cursor,
            matches!(self.focus, Focus::Facets),
        );

        let (list_rect, detail_rect) =
            split_content_with_detail(layout.content, !self.visible.is_empty());
        EventsPanel::render(
            frame,
            list_rect,
            &self.events,
            &self.visible,
            self.event_cursor,
            matches!(self.focus, Focus::Events),
        );
        if let Some(detail_rect) = detail_rect {
            let ev = self
                .visible
                .get(self.event_cursor)
                .map(|&i| &self.events[i as usize]);
            DetailPanel::render(frame, detail_rect, ev, self.source.as_deref());
        }

        let toast = self.toast.as_ref().and_then(|(msg, is_err, at)| {
            if at.elapsed() < TOAST_TTL {
                Some((msg.as_str(), *is_err))
            } else {
                None
            }
        });
        statusbar::render(
            frame,
            layout.status,
            statusbar::StatusLine {
                format: self.format,
                total: self.events.len(),
                visible: self.visible.len(),
                path: self
                    .path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("?"),
                loading: self.loading,
                cancelled: self.cancelled,
                toast,
            },
        );

        if self.search.active {
            let rect = crate::ui::layout::centered(area, 60, 3);
            frame.render_widget(ratatui::widgets::Clear, rect);
            self.search.render(frame, rect);
        }
        if self.show_daterange {
            self.date_modal.render(frame, area);
        }
        if self.show_export {
            self.export_menu.render(frame, area);
        }
        if self.show_help {
            help_popup::render(frame, area);
        }
    }
}
