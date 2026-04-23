use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    Help,
    Up,
    Down,
    PageUp,
    PageDown,
    Top,
    Bottom,
    FocusFacets,
    FocusEvents,
    ToggleSelection,
    OpenSearch,
    OpenDateRange,
    OpenDetail,
    CloseOverlay,
    ResetFilters,
    ExportFacet,
}

pub struct KeyMap;

impl KeyMap {
    pub fn map(ev: KeyEvent) -> Option<Action> {
        let m = ev.modifiers;
        match ev.code {
            KeyCode::Char('q') => Some(Action::Quit),
            KeyCode::Char('c') if m.contains(KeyModifiers::CONTROL) => Some(Action::Quit),
            KeyCode::Char('?') => Some(Action::Help),
            KeyCode::Char('j') | KeyCode::Down => Some(Action::Down),
            KeyCode::Char('k') | KeyCode::Up => Some(Action::Up),
            KeyCode::PageDown => Some(Action::PageDown),
            KeyCode::PageUp => Some(Action::PageUp),
            KeyCode::Char('g') => Some(Action::Top),
            KeyCode::Char('G') => Some(Action::Bottom),
            KeyCode::Char('f') => Some(Action::FocusFacets),
            KeyCode::Char('e') => Some(Action::FocusEvents),
            KeyCode::Char(' ') => Some(Action::ToggleSelection),
            KeyCode::Char('/') => Some(Action::OpenSearch),
            KeyCode::Char('d') => Some(Action::OpenDateRange),
            KeyCode::Char('r') => Some(Action::ResetFilters),
            KeyCode::Char('x') => Some(Action::ExportFacet),
            KeyCode::Enter => Some(Action::OpenDetail),
            KeyCode::Esc => Some(Action::CloseOverlay),
            _ => None,
        }
    }
}

pub struct Binding {
    pub keys: &'static str,
    pub desc: &'static str,
}

pub const HELP: &[Binding] = &[
    Binding {
        keys: "q / Ctrl-C",
        desc: "quit",
    },
    Binding {
        keys: "?",
        desc: "toggle help",
    },
    Binding {
        keys: "j / k, ↓ / ↑",
        desc: "move cursor",
    },
    Binding {
        keys: "g / G",
        desc: "top / bottom",
    },
    Binding {
        keys: "PgUp / PgDn",
        desc: "page",
    },
    Binding {
        keys: "f / e",
        desc: "focus Facets / Events",
    },
    Binding {
        keys: "Space",
        desc: "toggle facet value",
    },
    Binding {
        keys: "/",
        desc: "text search",
    },
    Binding {
        keys: "d",
        desc: "date range modal",
    },
    Binding {
        keys: "Enter",
        desc: "open event detail",
    },
    Binding {
        keys: "r",
        desc: "reset filters",
    },
    Binding {
        keys: "x",
        desc: "export menu (facet group or filtered log)",
    },
    Binding {
        keys: "Esc",
        desc: "close overlay / clear search",
    },
];
